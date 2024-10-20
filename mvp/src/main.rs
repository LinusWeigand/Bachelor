use axum::extract::DefaultBodyLimit;

mod handlers;
mod route;

const PARQUET_FOLDER: &str = "./parquet_files/";

#[tokio::main]
async fn main() {
    let app = route::create_router()
    .layer(DefaultBodyLimit::max(40 * 1024 * 1024));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
