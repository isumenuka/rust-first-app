//! End-to-end tests of the HTTP API.
//!
//! Requests go straight into the router with `oneshot`, so no port is opened.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use rust_first_app::app;

/// Sends one request and returns the status plus the JSON body (`Null` if empty).
async fn send(app: &Router, method: &str, uri: &str, body: Option<&Value>) -> (StatusCode, Value) {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(json) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string())),
        None => builder.body(Body::empty()),
    }
    .expect("test request should be valid");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("router never fails");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body should be readable")
        .to_bytes();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("body should be JSON")
    };
    (status, json)
}

#[tokio::test]
async fn index_page_is_served_as_html() {
    let request = Request::get("/").body(Body::empty()).unwrap();

    let response = app().oneshot(request).await.unwrap();

    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert_eq!(content_type, "text/html; charset=utf-8");
}

#[tokio::test]
async fn list_todos_is_empty_for_new_app() {
    let app = app();

    let (_, body) = send(&app, "GET", "/api/todos", None).await;

    assert_eq!(body, json!([]));
}

#[tokio::test]
async fn create_todo_responds_with_201() {
    let app = app();

    let (status, _) = send(
        &app,
        "POST",
        "/api/todos",
        Some(&json!({ "title": "Learn Rust" })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn create_todo_returns_the_new_todo() {
    let app = app();

    let (_, body) = send(
        &app,
        "POST",
        "/api/todos",
        Some(&json!({ "title": "Learn Rust" })),
    )
    .await;

    assert_eq!(
        body,
        json!({ "id": 1, "title": "Learn Rust", "done": false })
    );
}

#[tokio::test]
async fn created_todo_shows_up_in_the_list() {
    let app = app();
    send(
        &app,
        "POST",
        "/api/todos",
        Some(&json!({ "title": "Learn Rust" })),
    )
    .await;

    let (_, body) = send(&app, "GET", "/api/todos", None).await;

    assert_eq!(
        body,
        json!([{ "id": 1, "title": "Learn Rust", "done": false }])
    );
}

#[tokio::test]
async fn create_todo_rejects_blank_title_with_422() {
    let app = app();

    let (status, _) = send(&app, "POST", "/api/todos", Some(&json!({ "title": "   " }))).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_todo_explains_why_title_was_rejected() {
    let app = app();

    let (_, body) = send(&app, "POST", "/api/todos", Some(&json!({ "title": "" }))).await;

    assert_eq!(body, json!({ "error": "title must not be empty" }));
}

#[tokio::test]
async fn toggle_todo_marks_it_done() {
    let app = app();
    send(
        &app,
        "POST",
        "/api/todos",
        Some(&json!({ "title": "Learn Rust" })),
    )
    .await;

    let (_, body) = send(&app, "POST", "/api/todos/1/toggle", None).await;

    assert_eq!(
        body,
        json!({ "id": 1, "title": "Learn Rust", "done": true })
    );
}

#[tokio::test]
async fn toggle_unknown_todo_responds_with_404() {
    let app = app();

    let (status, _) = send(&app, "POST", "/api/todos/99/toggle", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_todo_responds_with_204() {
    let app = app();
    send(
        &app,
        "POST",
        "/api/todos",
        Some(&json!({ "title": "Learn Rust" })),
    )
    .await;

    let (status, _) = send(&app, "DELETE", "/api/todos/1", None).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn deleted_todo_is_gone_from_the_list() {
    let app = app();
    send(
        &app,
        "POST",
        "/api/todos",
        Some(&json!({ "title": "Learn Rust" })),
    )
    .await;
    send(&app, "DELETE", "/api/todos/1", None).await;

    let (_, body) = send(&app, "GET", "/api/todos", None).await;

    assert_eq!(body, json!([]));
}
