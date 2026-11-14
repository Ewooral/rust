//! One feature through every layer: repository → service → handler (via the router).
//! Run: cargo run -p capstone_rest_api --example layers_in_action
//!
//! Each layer is called directly, to show exactly what it is responsible for.

use axum::body::Body;
use axum::http::{Request, header};
use capstone_rest_api::auth::AuthUser;
use capstone_rest_api::domain::{NewReview, RegisterRequest, Role};
use capstone_rest_api::repository::ReviewRepository;
use capstone_rest_api::{AppState, Config, build_router};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::build(Config::for_tests()).await?;
    let ama = state
        .auth
        .register(RegisterRequest {
            email: "ama@example.com".into(),
            password: "correct horse battery".into(),
            display_name: "Ama".into(),
        })
        .await?;

    println!("== 1. repository: SQL in, rows out. No rules at all.");
    let repo = ReviewRepository::new(state.db.clone());
    let nonsense = NewReview {
        book_title: "Emma".into(),
        rating: 5,
        body: String::new(),
    };
    let review = repo.insert(ama.id, &nonsense).await?;
    println!("inserted review {} by {}", review.id, review.author_name);
    // The database's CHECK constraint is the last line of defence:
    let bad = NewReview {
        rating: 9,
        ..nonsense.clone()
    };
    println!(
        "rating 9 straight into SQL: {}",
        repo.insert(ama.id, &bad).await.unwrap_err()
    );

    println!("\n== 2. service: validation and authorisation, in AppError terms.");
    println!(
        "rating 9 via the service:   {:?}",
        state
            .reviews
            .create(
                &AuthUser {
                    id: ama.id,
                    role: Role::Member
                },
                bad
            )
            .await
            .unwrap_err()
    );
    let stranger = AuthUser {
        id: 999,
        role: Role::Member,
    };
    println!(
        "a stranger deletes it:      {:?}",
        state
            .reviews
            .delete(&stranger, review.id)
            .await
            .unwrap_err()
    );

    println!("\n== 3. handler + router: HTTP status and JSON body.");
    let token = {
        let login = capstone_rest_api::domain::LoginRequest {
            email: "ama@example.com".into(),
            password: "correct horse battery".into(),
        };
        state.auth.login(login).await?.access_token
    };
    let app = build_router(state);
    let request = Request::post("/api/v1/reviews")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"book_title":"Emma","rating":9}"#))?;
    let response = app.oneshot(request).await?;
    let status = response.status();
    let body = response.into_body().collect().await?.to_bytes();
    println!("POST /api/v1/reviews -> {status}");
    println!("{}", String::from_utf8_lossy(&body));
    Ok(())
}
