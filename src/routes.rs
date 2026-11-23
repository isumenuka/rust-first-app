//! HTTP layer: routes, handlers and the mapping from errors to responses.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::todos::{Todo, TodoError, TodoList};

// tokio's RwLock (not std's): waiting for it doesn't block the runtime's threads,
// and it cannot be poisoned, so handlers never need to unwrap a lock result.
type SharedList = Arc<RwLock<TodoList>>;

const INDEX_HTML: &str = include_str!("../static/index.html");

/// Builds the app's router with a fresh, empty to-do list.
///
/// | Method   | Path                     | Does                                  |
/// |----------|--------------------------|---------------------------------------|
/// | `GET`    | `/`                      | Serves the web page                   |
/// | `GET`    | `/api/todos`             | Lists all to-dos                      |
/// | `POST`   | `/api/todos`             | Creates one from `{"title": "..."}`   |
/// | `POST`   | `/api/todos/{id}/toggle` | Flips its done state                  |
/// | `DELETE` | `/api/todos/{id}`        | Removes it                            |
///
/// Errors come back as `{"error": "..."}` with status 404 (unknown id)
/// or 422 (invalid title).
pub fn app() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/todos", get(list_todos).post(create_todo))
        .route("/api/todos/{id}/toggle", post(toggle_todo))
        .route("/api/todos/{id}", delete(delete_todo))
        .with_state(SharedList::default())
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn list_todos(State(list): State<SharedList>) -> Response {
    let guard = list.read().await;
    // Serialize while holding the read lock instead of cloning the whole list.
    Json(guard.all()).into_response()
}

#[derive(Debug, Deserialize)]
struct NewTodo {
    title: String,
}

async fn create_todo(
    State(list): State<SharedList>,
    Json(new_todo): Json<NewTodo>,
) -> Result<(StatusCode, Json<Todo>), TodoError> {
    let todo = list.write().await.add(&new_todo.title)?;
    Ok((StatusCode::CREATED, Json(todo)))
}

async fn toggle_todo(
    State(list): State<SharedList>,
    Path(id): Path<u64>,
) -> Result<Json<Todo>, TodoError> {
    let todo = list.write().await.toggle(id)?;
    Ok(Json(todo))
}

async fn delete_todo(
    State(list): State<SharedList>,
    Path(id): Path<u64>,
) -> Result<StatusCode, TodoError> {
    list.write().await.remove(id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for TodoError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::EmptyTitle | Self::TitleTooLong => StatusCode::UNPROCESSABLE_ENTITY,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
        };
        let body = ErrorBody {
            error: self.to_string(),
        };
        (status, Json(body)).into_response()
    }
}
