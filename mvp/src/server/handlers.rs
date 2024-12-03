use actix_web::{error::ErrorInternalServerError, web, Error, HttpResponse, Responder};
use futures::StreamExt;
use serde_json::json;
use std::path::PathBuf;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncWriteExt;

use crate::{MAX_CHUNK_SIZE, PARQUET_FOLDER};

pub async fn health_checker_handler() -> impl Responder {
    let response = json!({
        "status": "success",
        "message": "API is running"
    });
    HttpResponse::Ok().json(response)
}

pub async fn get_parquet_file(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let file_name = path.into_inner();
    let file_path = PathBuf::from(PARQUET_FOLDER).join(&file_name);

    if !file_path.exists() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let file = File::open(&file_path)
        .await
        .map_err(ErrorInternalServerError)?;
    let file_stream = tokio_util::io::ReaderStream::new(file);

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .streaming(file_stream))
}

pub async fn put_parquet_file(
    path: web::Path<String>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    // let file_name = path.into_inner();

    // create_dir_all(PARQUET_FOLDER)
    //     .await
    //     .map_err(|e| ErrorInternalServerError(format!("Failed to create directory: {e}")))?;

    // let full_path = PathBuf::from(PARQUET_FOLDER).join(&file_name);

    // let mut file = File::create(&full_path)
    //     .await
    //     .map_err(|e| ErrorInternalServerError(format!("Failed to create file: {e}")))?;

    let mut buffer = Vec::new();

    while let Some(chunk) = payload.next().await {
        let data =
            chunk.map_err(|e| ErrorInternalServerError(format!("Failed to read chunk: {e}")))?;

        buffer.extend_from_slice(&data);

        while buffer.len() >= MAX_CHUNK_SIZE {
            let chunk_to_write = buffer.drain(..MAX_CHUNK_SIZE).collect::<Vec<_>>();

            if let Some(x) = chunk_to_write.first() {
                println!("{},{}", x, path);
            }
            // file.write_all(&chunk_to_write)
            //     .await
            //     .map_err(|e| ErrorInternalServerError(format!("Failed to write chunk: {e}")))?;
        }
    }
    if !buffer.is_empty() {
        // file.write_all(&buffer)
        //     .await
        //     .map_err(|e| ErrorInternalServerError(format!("Failed to write final chunk: {e}")))?;
    }

    // file.flush()
    //     .await
    //     .map_err(|e| ErrorInternalServerError(format!("Failed to flush file: {e}")))?;

    Ok(HttpResponse::Ok().finish())
}
