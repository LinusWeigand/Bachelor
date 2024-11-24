use std::{net::SocketAddr, time::Duration};

use axum::{extract::DefaultBodyLimit, Router};
use hyper::{server::conn::http1, service::service_fn, Request};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use tokio::{net::{TcpListener, TcpStream}, runtime::Builder, time::timeout};

mod handlers;
mod route;

const PARQUET_FOLDER: &str = "/mnt/raid0/";
const IDLE_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_BUF_SIZE: usize = 16 * 1024;

// fn main() {
//     let runtime = Builder::new_current_thread().enable_all().build().unwrap();
//
//     runtime.block_on(async {
//         let app = route::create_router()
//             .layer(DefaultBodyLimit::max(40 * 1024 * 1024));
//
//         let hyper_app = TowerToHyperService::new(app.clone());
//         let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
//
//         loop {
//             let (socket, _) = listener.accept().await.unwrap();
//
//             let hyper_app = hyper_app.clone();
//
//             tokio::spawn(async move {
//                 match timeout(IDLE_TIMEOUT, handle_connection(socket, hyper_app)).await {
//                     Ok(Ok(())) => {} 
//                     Ok(Err(e)) => eprintln!("Connection error: {}", e),
//                     Err(_) => eprintln!("Connection timed out"),
//                 }
//             });
//         }
//     });
// }
//
// pub async fn handle_connection(socket: TcpStream, hyper_app: TowerToHyperService<Router>) -> Result<(), hyper::Error>{
//     let stream = TokioIo::new(socket);
//     let mut builder = http1::Builder::new();
//     let http = builder
//         .keep_alive(true)
//         .max_buf_size(MAX_BUF_SIZE);
//     http.serve_connection(stream, hyper_app).await
// }

#[tokio::main]
async fn main() {
    let app = route::create_router().layer(DefaultBodyLimit::max(40 * 1024 * 1024));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
