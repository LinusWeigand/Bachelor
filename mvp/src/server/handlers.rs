use std::path::PathBuf;

use axum::{
    body::Body,
    extract::{Multipart, Path},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};
use tokio_util::io::ReaderStream;

use crate::PARQUET_FOLDER;

pub async fn health_checker_handler() -> impl IntoResponse {
    let response = json!({
        "status": "success",
        "message": "API is running"
    });
    Json(response)
}

pub async fn get_parquet_file(
    Path(file_name): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let file_path = PathBuf::from(PARQUET_FOLDER).join(&*file_name);

    match file_path.try_exists() {
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(false) => return Err(StatusCode::NOT_FOUND),
        Ok(true) => {}
    };

    let file = match File::open(&file_path).await {
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(f) => f,
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    return match Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(body)
    {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(r) => Ok(r),
    };
}

pub async fn put_parquet_file(
    Path(file_name): Path<String>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    if create_dir_all(PARQUET_FOLDER).await.is_err() {
        eprintln!("Failed to create directory: {}", PARQUET_FOLDER);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let full_path = PathBuf::from(PARQUET_FOLDER).join(&file_name);

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| {
            eprintln!("Failed to read next multipart field: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    {
        if let Some(_file_name_in_field) = field.file_name() {

            let mut file = tokio::fs::File::create(&full_path).await.map_err(|e| {
                eprintln!("Failed to create file {}: {e}", full_path.display());
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let mut field_data = field;

            while let Some(chunk) = field_data
                .chunk()
                .await
                .map_err(|e| {
                    eprintln!("Failed to read field chunk: {e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })? 
            {
                if let Err(e) = file.write_all(&chunk).await {
                    eprintln!("Failed to write chunk to file {}: {e}", full_path.display());
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }

            file.flush().await.map_err(|e| {
                eprintln!("Failed to flush file {}: {e}", full_path.display());
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            return Ok(StatusCode::OK);
        } else {
            eprintln!("Multipart field is missing a file name");
            return Err(StatusCode::BAD_REQUEST);
        };
    }
    Err(StatusCode::BAD_REQUEST)
}
