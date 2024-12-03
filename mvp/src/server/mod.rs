use actix_web::{App, HttpServer};

mod routes;
mod handlers;

const MAX_CHUNK_SIZE: usize = 8192;
const PARQUET_FOLDER: &str = "/mnt/raid0/";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .configure(routes::init_routes) 
    })
    .bind("0.0.0.0:80")?
    .run()
    .await
}

