use axum::{Json, Router, routing::get};
use serde::Serialize;

use crate::todo;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub fn create_app() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .nest("/todos", todo::routes())
}
