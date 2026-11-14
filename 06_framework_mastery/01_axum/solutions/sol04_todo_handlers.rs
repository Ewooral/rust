//! Solution to exercise 4: write a router and three handlers (Path, Query, Json, State).
//!
//! The state is a list of todos behind `Arc<Mutex<..>>`. Implement:
//!
//! - `list_todos`, `GET /todos?done=true&limit=2`: the todos in id order,
//!   keeping only those whose `done` matches (if `done` is given), and at
//!   most `limit` of them (if `limit` is given).
//! - `get_todo`, `GET /todos/{id}`: `200` and the todo as JSON, or `404`
//!   with the plain-text body `todo 9 not found`.
//! - `create_todo`, `POST /todos` with `{"title":"..."}`: give it the next
//!   id (largest id + 1, or 1 for an empty list), `done: false`, store it,
//!   and answer `201 Created` with the new todo as JSON. A title that is
//!   empty after trimming gets `422` and the body `title must not be empty`.
//! - `app`: the router with those three routes and the state.
//!
//! Hints: `Router::new().route(path, get(a).post(b))`, `.with_state(state)`;
//! a handler can return `Result<Json<Todo>, (StatusCode, String)>`, or
//! `(StatusCode, Json<Todo>)`; lock the mutex with `.lock().unwrap()`.
//!
//! Test: cargo test -p axum_basics --features solutions --bin sol04_todo_handlers

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}

#[derive(Deserialize)]
struct NewTodo {
    title: String,
}

#[derive(Deserialize)]
struct Filter {
    done: Option<bool>,
    limit: Option<usize>,
}

#[derive(Clone)]
struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
}

async fn list_todos(
    State(state): State<AppState>,
    Query(filter): Query<Filter>,
) -> Json<Vec<Todo>> {
    let todos = state.todos.lock().unwrap();
    let list = todos
        .iter()
        .filter(|t| filter.done.is_none_or(|done| t.done == done))
        .take(filter.limit.unwrap_or(usize::MAX))
        .cloned()
        .collect();
    Json(list)
}

async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<Todo>, (StatusCode, String)> {
    let todos = state.todos.lock().unwrap();
    match todos.iter().find(|t| t.id == id) {
        Some(todo) => Ok(Json(todo.clone())),
        None => Err((StatusCode::NOT_FOUND, format!("todo {id} not found"))),
    }
}

async fn create_todo(
    State(state): State<AppState>,
    Json(new): Json<NewTodo>,
) -> Result<(StatusCode, Json<Todo>), (StatusCode, String)> {
    let title = new.title.trim();
    if title.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "title must not be empty".to_string(),
        ));
    }
    let mut todos = state.todos.lock().unwrap();
    let id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let todo = Todo {
        id,
        title: title.to_string(),
        done: false,
    };
    todos.push(todo.clone());
    Ok((StatusCode::CREATED, Json(todo)))
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/{id}", get(get_todo))
        .with_state(state)
}

fn sample_state() -> AppState {
    let todos = vec![
        Todo {
            id: 1,
            title: "learn routing".into(),
            done: true,
        },
        Todo {
            id: 2,
            title: "learn extractors".into(),
            done: false,
        },
        Todo {
            id: 3,
            title: "learn state".into(),
            done: false,
        },
    ];
    AppState {
        todos: Arc::new(Mutex::new(todos)),
    }
}

#[tokio::main]
async fn main() {
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use tower::ServiceExt;

    let app = app(sample_state());
    for uri in [
        "/todos",
        "/todos?done=false&limit=1",
        "/todos/2",
        "/todos/9",
    ] {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        println!("GET {uri} -> {status} {}", String::from_utf8_lossy(&body));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, header};
    use tower::ServiceExt;

    async fn send(app: &Router, request: Request<Body>) -> (StatusCode, String) {
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    fn ids(json: &str) -> Vec<u32> {
        let todos: Vec<Todo> = serde_json::from_str(json).unwrap();
        todos.iter().map(|t| t.id).collect()
    }

    #[tokio::test]
    async fn lists_filters_and_limits() {
        let app = app(sample_state());
        let (status, body) = send(&app, get("/todos")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ids(&body), [1, 2, 3]);
        assert_eq!(ids(&send(&app, get("/todos?done=false")).await.1), [2, 3]);
        assert_eq!(ids(&send(&app, get("/todos?done=true")).await.1), [1]);
        assert_eq!(ids(&send(&app, get("/todos?limit=2")).await.1), [1, 2]);
        assert_eq!(
            ids(&send(&app, get("/todos?done=false&limit=1")).await.1),
            [2]
        );
    }

    #[tokio::test]
    async fn gets_one_or_404() {
        let app = app(sample_state());
        let (status, body) = send(&app, get("/todos/2")).await;
        assert_eq!(status, StatusCode::OK);
        let todo: Todo = serde_json::from_str(&body).unwrap();
        assert_eq!(todo.title, "learn extractors");
        assert_eq!(
            send(&app, get("/todos/9")).await,
            (StatusCode::NOT_FOUND, "todo 9 not found".to_string())
        );
    }

    #[tokio::test]
    async fn creates_with_the_next_id_and_validates() {
        let app = app(sample_state());
        let post = |json: &str| {
            Request::post("/todos")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json.to_string()))
                .unwrap()
        };
        let (status, body) = send(&app, post(r#"{"title":"write tests"}"#)).await;
        assert_eq!(status, StatusCode::CREATED);
        let created: Todo = serde_json::from_str(&body).unwrap();
        assert_eq!(
            created,
            Todo {
                id: 4,
                title: "write tests".into(),
                done: false
            }
        );
        assert_eq!(send(&app, get("/todos/4")).await.0, StatusCode::OK);
        assert_eq!(
            send(&app, post(r#"{"title":"  "}"#)).await,
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                "title must not be empty".to_string()
            )
        );
    }
}
