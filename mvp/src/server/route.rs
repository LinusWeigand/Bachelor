use axum::{routing::get, Router};

use super::handlers::{get_parquet_file, health_checker_handler, put_parquet_file};

pub fn create_router() -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route(
            "/parquet/:file_name",
            get(get_parquet_file).put(put_parquet_file),
        )
}
