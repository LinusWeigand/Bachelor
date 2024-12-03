use crate::handlers;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/healthchecker").route(web::get().to(handlers::health_checker_handler)),
    )
    .service(
        web::resource("/parquet/{file_name}")
            .route(web::get().to(handlers::get_parquet_file))
            .route(web::put().to(handlers::put_parquet_file)),
    );
}
