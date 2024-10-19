use axum::{
    routing::{get},
    Router,
};
use std::sync::Arc;

use crate::{handlers::health_checker_handler, AppState};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .with_state(app_state)
}
