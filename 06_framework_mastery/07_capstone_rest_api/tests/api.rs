//! Handler tests: the whole router (middleware, handlers, services,
//! repositories, a real in-memory SQLite database), driven with
//! `tower::ServiceExt::oneshot`. No network, no ports.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use capstone_rest_api::{Config, build_app};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

/// A fresh app with its own empty database.
async fn app() -> Router {
    build_app(Config::for_tests())
        .await
        .expect("the app starts")
}

struct Reply {
    status: StatusCode,
    json: Value,
    location: Option<String>,
    request_id: Option<String>,
}

async fn call(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Reply {
    let mut request = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let request = match body {
        Some(body) => request
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string())),
        None => request.body(Body::empty()),
    }
    .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let text = |name| {
        response
            .headers()
            .get(name)
            .map(|v: &header::HeaderValue| v.to_str().unwrap().to_string())
    };
    let location = text(header::LOCATION.as_str());
    let request_id = text("x-request-id");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Reply {
        status,
        json,
        location,
        request_id,
    }
}

/// Registers and logs in; returns the access token.
async fn sign_up(app: &Router, email: &str, name: &str) -> String {
    let password = "correct horse battery staple";
    let reply = call(
        app,
        "POST",
        "/api/v1/auth/register",
        None,
        Some(json!({ "email": email, "password": password, "display_name": name })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.json);
    let login = call(
        app,
        "POST",
        "/api/v1/auth/login",
        None,
        Some(json!({ "email": email, "password": password })),
    )
    .await;
    assert_eq!(login.status, StatusCode::OK, "{}", login.json);
    login.json["access_token"].as_str().unwrap().to_string()
}

async fn post_review(app: &Router, token: &str, title: &str, rating: i64) -> i64 {
    let reply = call(
        app,
        "POST",
        "/api/v1/reviews",
        Some(token),
        Some(json!({ "book_title": title, "rating": rating, "body": "worth reading" })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.json);
    reply.json["id"].as_i64().unwrap()
}

#[tokio::test]
async fn health_reports_the_database() {
    let app = app().await;
    let reply = call(&app, "GET", "/health", None, None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json["status"], "ok");
    assert_eq!(reply.json["database"], "ok");
    assert!(
        reply.request_id.is_some(),
        "every response carries x-request-id"
    );
}

#[tokio::test]
async fn register_login_and_me() {
    let app = app().await;
    let token = sign_up(&app, "Ama@Example.com", "Ama").await;
    let me = call(&app, "GET", "/api/v1/me", Some(&token), None).await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.json["email"], "ama@example.com");
    assert_eq!(me.json["role"], "member");
    assert!(
        me.json.get("password_hash").is_none(),
        "never leak the hash"
    );
}

#[tokio::test]
async fn registration_is_validated_and_unique() {
    let app = app().await;
    let bad = call(
        &app,
        "POST",
        "/api/v1/auth/register",
        None,
        Some(json!({ "email": "nope", "password": "short", "display_name": "" })),
    )
    .await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bad.json["error"]["code"], "validation_failed");
    let fields: Vec<&str> = bad.json["error"]["details"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, ["email", "password", "display_name"]);

    sign_up(&app, "kofi@example.com", "Kofi").await;
    let again = call(
        &app,
        "POST",
        "/api/v1/auth/register",
        None,
        Some(json!({ "email": "KOFI@example.com", "password": "another long password", "display_name": "K" })),
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert_eq!(again.json["error"]["code"], "conflict");
}

#[tokio::test]
async fn wrong_credentials_get_the_same_answer() {
    let app = app().await;
    sign_up(&app, "esi@example.com", "Esi").await;
    for (email, password) in [
        ("esi@example.com", "the wrong password"),
        ("nobody@example.com", "the wrong password"),
    ] {
        let reply = call(
            &app,
            "POST",
            "/api/v1/auth/login",
            None,
            Some(json!({ "email": email, "password": password })),
        )
        .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
        assert_eq!(
            reply.json["error"]["message"],
            "wrong e-mail address or password"
        );
    }
}

#[tokio::test]
async fn protected_routes_need_a_valid_token() {
    let app = app().await;
    let none = call(&app, "GET", "/api/v1/me", None, None).await;
    assert_eq!(none.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        none.json["error"]["message"],
        "missing Authorization header"
    );

    let forged = call(&app, "GET", "/api/v1/me", Some("not.a.token"), None).await;
    assert_eq!(forged.status, StatusCode::UNAUTHORIZED);
    assert_eq!(forged.json["error"]["message"], "the token is not valid");

    let create = call(
        &app,
        "POST",
        "/api/v1/reviews",
        None,
        Some(json!({ "book_title": "Emma", "rating": 5 })),
    )
    .await;
    assert_eq!(create.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_read_update_delete_a_review() {
    let app = app().await;
    let token = sign_up(&app, "yaw@example.com", "Yaw").await;
    let created = call(
        &app,
        "POST",
        "/api/v1/reviews",
        Some(&token),
        Some(json!({ "book_title": "  Treasure Island ", "rating": 4 })),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let id = created.json["id"].as_i64().unwrap();
    assert_eq!(created.location, Some(format!("/api/v1/reviews/{id}")));
    assert_eq!(created.json["book_title"], "Treasure Island");
    assert_eq!(created.json["author_name"], "Yaw");

    let fetched = call(&app, "GET", &format!("/api/v1/reviews/{id}"), None, None).await;
    assert_eq!(fetched.json, created.json, "reading is public");

    let patched = call(
        &app,
        "PATCH",
        &format!("/api/v1/reviews/{id}"),
        Some(&token),
        Some(json!({ "rating": 5 })),
    )
    .await;
    assert_eq!(patched.status, StatusCode::OK);
    assert_eq!(patched.json["rating"], 5);
    assert_eq!(patched.json["book_title"], "Treasure Island");

    let deleted = call(
        &app,
        "DELETE",
        &format!("/api/v1/reviews/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let gone = call(&app, "GET", &format!("/api/v1/reviews/{id}"), None, None).await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    assert_eq!(
        gone.json["error"]["message"],
        format!("review {id} not found")
    );
}

#[tokio::test]
async fn only_the_author_or_an_admin_may_change_a_review() {
    let app = app().await;
    let author = sign_up(&app, "abena@example.com", "Abena").await;
    let stranger = sign_up(&app, "kwame@example.com", "Kwame").await;
    let admin = sign_up(&app, "admin@example.com", "Admin").await; // in ADMIN_EMAILS
    let id = post_review(&app, &author, "Emma", 3).await;
    let path = format!("/api/v1/reviews/{id}");

    let refused = call(
        &app,
        "PATCH",
        &path,
        Some(&stranger),
        Some(json!({ "rating": 1 })),
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_eq!(refused.json["error"]["code"], "forbidden");
    let refused = call(&app, "DELETE", &path, Some(&stranger), None).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);

    let me = call(&app, "GET", "/api/v1/me", Some(&admin), None).await;
    assert_eq!(me.json["role"], "admin");
    let moderated = call(
        &app,
        "PATCH",
        &path,
        Some(&admin),
        Some(json!({ "body": "[removed]" })),
    )
    .await;
    assert_eq!(moderated.status, StatusCode::OK);
    let deleted = call(&app, "DELETE", &path, Some(&admin), None).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn invalid_reviews_and_patches_are_422() {
    let app = app().await;
    let token = sign_up(&app, "efua@example.com", "Efua").await;
    let bad = call(
        &app,
        "POST",
        "/api/v1/reviews",
        Some(&token),
        Some(json!({ "book_title": "", "rating": 9 })),
    )
    .await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bad.json["error"]["details"].as_array().unwrap().len(), 2);

    let id = post_review(&app, &token, "Dracula", 2).await;
    let bad_patch = call(
        &app,
        "PATCH",
        &format!("/api/v1/reviews/{id}"),
        Some(&token),
        Some(json!({ "rating": 0 })),
    )
    .await;
    assert_eq!(bad_patch.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn unreadable_input_keeps_axums_status_in_our_format() {
    let app = app().await;
    let token = sign_up(&app, "kojo@example.com", "Kojo").await;
    let unknown_field = call(
        &app,
        "POST",
        "/api/v1/reviews",
        Some(&token),
        Some(json!({ "book_title": "Emma", "rating": 5, "stars": 5 })),
    )
    .await;
    assert_eq!(unknown_field.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(unknown_field.json["error"]["code"], "invalid_body");
    let message = unknown_field.json["error"]["message"].as_str().unwrap();
    assert!(message.contains("unknown field `stars`"), "{message}");

    let bad_id = call(&app, "GET", "/api/v1/reviews/seven", None, None).await;
    assert_eq!(bad_id.status, StatusCode::BAD_REQUEST);
    let bad_query = call(&app, "GET", "/api/v1/reviews?page=-1", None, None).await;
    assert_eq!(bad_query.status, StatusCode::BAD_REQUEST);
    let bad_rating = call(&app, "GET", "/api/v1/reviews?min_rating=9", None, None).await;
    assert_eq!(bad_rating.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn lists_are_paginated_filtered_and_newest_first() {
    let app = app().await;
    let token = sign_up(&app, "adwoa@example.com", "Adwoa").await;
    for (title, rating) in [
        ("Emma", 5),
        ("Persuasion", 4),
        ("Dracula", 2),
        ("Emma", 3),
        ("Frankenstein", 5),
    ] {
        post_review(&app, &token, title, rating).await;
    }
    let first = call(&app, "GET", "/api/v1/reviews?per_page=2", None, None).await;
    assert_eq!(first.status, StatusCode::OK);
    assert_eq!(
        (
            first.json["total"].as_u64(),
            first.json["total_pages"].as_u64()
        ),
        (Some(5), Some(3))
    );
    let titles = |reply: &Reply| -> Vec<String> {
        reply.json["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["book_title"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(titles(&first), ["Frankenstein", "Emma"], "newest first");
    let last = call(&app, "GET", "/api/v1/reviews?per_page=2&page=3", None, None).await;
    assert_eq!(titles(&last), ["Emma"]);

    let emma = call(&app, "GET", "/api/v1/reviews?book=emm", None, None).await;
    assert_eq!(emma.json["total"], 2);
    let great = call(&app, "GET", "/api/v1/reviews?min_rating=5", None, None).await;
    assert_eq!(titles(&great), ["Frankenstein", "Emma"]);
}

#[tokio::test]
async fn unknown_routes_and_methods_answer_in_json() {
    let app = app().await;
    let missing = call(&app, "GET", "/api/v2/reviews", None, None).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(
        missing.json["error"]["message"],
        "route GET /api/v2/reviews not found"
    );
    let wrong_method = call(&app, "PUT", "/api/v1/reviews", None, None).await;
    assert_eq!(wrong_method.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(wrong_method.json["error"]["code"], "method_not_allowed");
}

#[tokio::test]
async fn bodies_over_the_limit_are_refused() {
    let app = app().await;
    let token = sign_up(&app, "nana@example.com", "Nana").await;
    let huge = "x".repeat(capstone_rest_api::routes::BODY_LIMIT + 1);
    let reply = call(
        &app,
        "POST",
        "/api/v1/reviews",
        Some(&token),
        Some(json!({ "book_title": "Big", "rating": 3, "body": huge })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(reply.json["error"]["code"], "payload_too_large");
}
