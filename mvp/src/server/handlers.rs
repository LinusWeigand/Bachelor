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

use super::PARQUET_FOLDER;

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
    if let Err(_) = create_dir_all(PARQUET_FOLDER).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        if let Some(_file_name_in_field) = field.file_name() {
            let full_path = PathBuf::from(PARQUET_FOLDER).join(&file_name);

            let mut file = match tokio::fs::File::create(full_path).await {
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                Ok(f) => f,
            };

            let content = field
                .bytes()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if let Err(_) = file.write_all(&content).await {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            return Ok(StatusCode::OK);
        };
    }
    Err(StatusCode::BAD_REQUEST)
}
