use axum::extract::DefaultBodyLimit;
use dotenv::dotenv;
use std::{env, sync::Arc};

mod route;
mod handlers;


pub struct AppState {
    url: String,
}

const PARQUET_FOLDER: &str = "./parquet_files/";

#[tokio::main]
async fn main() {
    dotenv().ok();
    let url = env::var("URL").expect("URL must be set!");
    let app = route::create_router(Arc::new(AppState {
        url,
    }))
    .layer(DefaultBodyLimit::max(40 * 1024 * 1024));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
