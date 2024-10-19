use axum::{response::IntoResponse, Json};
use serde_json::json;

pub async fn health_checker_handler() -> impl IntoResponse {
    let response = json!({
        "status": "success",
        "message": "API is running"
    });
    Json(response)
}
