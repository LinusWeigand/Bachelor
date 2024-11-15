use axum::extract::DefaultBodyLimit;
use tokio::runtime::Builder;

mod handlers;
mod route;

const PARQUET_FOLDER: &str = "./parquet_files/";

fn main() {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    runtime.block_on(async {
        let app = route::create_router().layer(DefaultBodyLimit::max(40 * 1024 * 1024));
        let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}
