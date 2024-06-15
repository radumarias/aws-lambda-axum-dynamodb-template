use std::{str::FromStr, time::SystemTime};

use axum::{
    extract::{Path, Query},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use uuid::Uuid;

#[derive(Deserialize)]
struct FilesQuery {
    after_date: DateTime<Utc>,
}

#[derive(Serialize)]
struct FilesResponse {
    files: Vec<File>,
}

#[derive(Serialize)]
struct File {
    file_id: Uuid,
    upload_date: DateTime<Utc>,
}

#[derive(Serialize)]
struct UploadResponse {
    upload_url: String,
}

#[derive(Serialize)]
struct AnalysisResponse {
    status: String,
    status_message: String,
    result_url: String,
}

#[derive(Serialize)]
struct AnalysisResultResponse {
    status: String,
    status_message: String,
    related_file_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
struct PathQuery {
    src: Uuid,
    dst: Uuid,
}

#[derive(Serialize)]
struct PathResponse {
    path: Vec<Uuid>,
}

async fn get_files(Path(_file_id): Path<Uuid>, _query: Query<FilesQuery>) -> Json<FilesResponse> {
    Json(FilesResponse {
        files: vec![
            File {
                file_id: Uuid::from_str("d7073ab3-10a2-47c4-a321-b4258c91fdb3").unwrap(),
                upload_date: DateTime::from(SystemTime::now()),
            },
            File {
                file_id: Uuid::from_str("d7073ab3-10a2-47c4-a321-b4258c91fdb1").unwrap(),
                upload_date: DateTime::from(SystemTime::now()),
            },
            File {
                file_id: Uuid::from_str("d7073ab3-10a2-47c4-a321-b4258c91fdb2").unwrap(),
                upload_date: DateTime::from(SystemTime::now()),
            },
        ],
    })
}

async fn post_upload(Path(_file_id): Path<Uuid>) -> Json<UploadResponse> {
    Json(UploadResponse {
        upload_url: "https://example.com/upload/42".to_string(),
    })
}

async fn get_analysis(Path(_file_id): Path<Uuid>) -> Json<AnalysisResponse> {
    Json(AnalysisResponse {
        status: "processing".to_string(),
        status_message: "".to_string(),
        result_url: "https://rust423.shuttleapp.rs/v1/results/d7073ab3-10a2-47c4-a321-b4258c91fdb3"
            .to_string(),
    })
}

async fn get_results(Path(_file_id): Path<Uuid>) -> Json<AnalysisResultResponse> {
    Json(AnalysisResultResponse {
        status: "processed".to_string(),
        status_message: "".to_string(),
        related_file_ids: vec![
            Uuid::from_str("d7073ab3-10a2-47c4-a321-b4258c91fdb3").unwrap(),
            Uuid::from_str("d7073ab3-10a2-47c4-a321-b4258c91fdb1").unwrap(),
            Uuid::from_str("d7073ab3-10a2-47c4-a321-b4258c91fdb2").unwrap(),
        ],
    })
}

async fn get_path(_query: Query<PathQuery>) -> Json<PathResponse> {
    Json(PathResponse {
        path: vec![
            Uuid::from_str("d7073ab3-10a2-47c4-a321-b4258c91fdb3").unwrap(),
            Uuid::from_str("d7073ab3-10a2-47c4-a321-b4258c91fdb1").unwrap(),
            Uuid::from_str("d7073ab3-10a2-47c4-a321-b4258c91fdb2").unwrap(),
        ],
    })
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/v1/upload/:file_id", post(post_upload))
        .route("/v1/analysis/:file_id", get(get_analysis))
        .route("/v1/results/:file_id", get(get_results))
        .route("/v1/files/:file_id", get(get_files))
        .route("/v1/path", get(get_path))
        .layer(CorsLayer::new().allow_origin(AllowOrigin::mirror_request()))
        .layer(TraceLayer::new_for_http());

    Ok(router.into())
}
