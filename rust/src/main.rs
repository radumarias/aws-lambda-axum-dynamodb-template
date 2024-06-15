use async_injector::{Injector, Provider};
use axum::{extract::State, routing::get, Router};
use lambda_http::{run, Error};
use lambda_web::is_running_on_lambda;
use std::{str::FromStr, time::SystemTime};

use axum::{
    extract::{Path, Query},
    routing::post,
    Json,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use uuid::Uuid;
use aws_sdk_dynamodb::Client;
use aws_config;
use aws_sdk_dynamodb::types::AttributeValue;

const TABLE: &str = "rust-test";

#[derive(Debug, Provider)]
struct Repository {
    #[dependency]
    client: Client,
}

#[derive(Debug, Deserialize)]
struct UploadRequest {
    hash: String,
}

#[derive(Debug, Serialize)]
struct UploadResponse {
    upload_url: String,
}

#[derive(Debug, Deserialize)]
struct AfterDateQuery {
    after_date: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct FilesResponse {
    files: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize)]
struct File {
    file_id: String,
    upload_date: DateTime<Utc>,
    hash: String,
}

#[derive(Debug, Serialize)]
struct AnalysisResponse {
    status: String,
    status_message: String,
    result_url: String,
}

#[derive(Debug, Serialize)]
struct AnalysisResultResponse {
    status: String,
    status_message: String,
    related_file_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
struct IdsSrcDstQuery {
    src: Uuid,
    dst: Uuid,
}

#[derive(Debug, Serialize)]
struct PathResponse {
    path: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: u64,
    per_page: usize,
}

async fn post_upload(
    State(injector): State<&'static Injector>,
    Path(file_id): Path<Uuid>,
    Json(payload): Json<UploadRequest>,
) -> Json<UploadResponse> {
    let client = Repository::provider(&injector)
        .await
        .unwrap()
        .wait()
        .await
        .client;

    let uuid_av = AttributeValue::S(file_id.to_string());
    let created_at_av = AttributeValue::N(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string());
    let hash_av = AttributeValue::S(payload.hash);

    let request = client
        .put_item()
        .table_name(TABLE)
        .item("uuid", uuid_av)
        .item("created_at", created_at_av)
        .item("hash", hash_av);

    println!("Executing request [{request:?}] to add item...");

    let resp = request.send().await.unwrap();

    // let attributes = resp.attributes().unwrap();
    //
    // let uuid = attributes.get("uuid").cloned();
    // let created_at = attributes.get("created_at").cloned();
    // let hash = attributes.get("hash").cloned();
    //
    // println!(
    //     "Added file {uuid:?}, {created_at:?}, {hash:?}"
    // );

    Json(UploadResponse {
        upload_url: "https://example.com/upload/42".to_string(),
    })
}

async fn get_analysis(
    State(injector): State<&'static Injector>,
    Path(file_id): Path<Uuid>,
) -> Json<AnalysisResponse> {
    let client = Repository::provider(&injector)
        .await
        .unwrap()
        .wait()
        .await
        .client;

    Json(AnalysisResponse {
        status: "processing".to_string(),
        status_message: "".to_string(),
        result_url: format!("https://ll09yudnr6.execute-api.us-east-1.amazonaws.com/v1/results/{file_id}"),
    })
}

async fn get_results(
    State(injector): State<&'static Injector>,
    Path(_file_id): Path<Uuid>,
    pagination: Query<PaginationQuery>,
) -> Json<AnalysisResultResponse> {
    let client = Repository::provider(&injector)
        .await
        .unwrap()
        .wait()
        .await
        .client;

    let page_size = pagination.per_page;
    let items: Result<Vec<_>, _> = client
        .scan()
        .table_name(TABLE)
        .limit(page_size as i32)
        .into_paginator()
        .items()
        .send()
        .collect()
        .await;

    let ids = items.unwrap().iter()
        .map(|item| Uuid::from_str(item.get("uuid").unwrap().as_s().unwrap()).unwrap())
        .collect::<Vec<_>>();

    Json(AnalysisResultResponse {
        status: "processed".to_string(),
        status_message: "".to_string(),
        related_file_ids: ids,
    })
}

async fn get_files(
    State(injector): State<&'static Injector>,
    Path(_file_id): Path<Uuid>,
    _after_date: Query<AfterDateQuery>,
    pagination: Query<PaginationQuery>,
) -> Json<FilesResponse> {
    let client = Repository::provider(&injector)
        .await
        .unwrap()
        .wait()
        .await
        .client;

    let page_size = pagination.per_page;
    let items: Result<Vec<_>, _> = client
        .scan()
        .table_name(TABLE)
        .limit(page_size as i32)
        .into_paginator()
        .items()
        .send()
        .collect()
        .await;

    let files = items.unwrap().iter().map(|item| {
        let uuid = item.get("uuid").unwrap().as_s().unwrap().to_string();
        let created_at = item.get("created_at").unwrap().as_n().unwrap().parse::<u64>().unwrap();
        let hash = item.get("hash").unwrap().as_s().unwrap().to_string();
        File {
            file_id: uuid,
            upload_date: DateTime::<Utc>::from_timestamp_nanos(created_at as i64 * 1_000_000_000),
            hash,
        }
    }).collect::<Vec<_>>();

    Json(FilesResponse { files })
}

async fn get_path(
    State(injector): State<&'static Injector>,
    ids: Query<IdsSrcDstQuery>,
    pagination: Query<PaginationQuery>,
) -> Json<PathResponse> {
    let client = Repository::provider(&injector)
        .await
        .unwrap()
        .wait()
        .await
        .client;

    let ids = match client
        .execute_statement()
        .statement(format!(
            r#"SELECT uuid FROM "{TABLE}" WHERE uuid IN [?, ?]"#
        ))
        .set_parameters(Some(vec![
            AttributeValue::S(ids.src.to_string()),
            AttributeValue::S(ids.dst.to_string()),
        ]))
        .send()
        .await
    {
        Ok(resp) => {
            if !resp.items().is_empty() {
                println!("Found {} matching entry in the table", resp.items.as_ref().unwrap().len());
                resp.items.unwrap().iter()
                    .map(|item| Uuid::from_str(item.get("uuid").unwrap().as_s().unwrap()).unwrap())
                    .collect()
            } else {
                println!("Did not find a match.");
                vec![]
            }
        }
        Err(e) => {
            println!("Got an error querying table:");
            println!("{}", e);
            panic!("Error querying table.");
        }
    };

    Json(PathResponse {
        path: ids,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let injector = Injector::new();
    injector.update(client).await;

    // let init_provider = InitProvider(provider);
    let injector: &'static Injector = Box::leak(Box::new(injector));

    // build our application with a route
    let app = Router::new()
        .route("/v1/upload/:file_id", post(post_upload))
        .route("/v1/analysis/:file_id", get(get_analysis))
        .route("/v1/results/:file_id", get(get_results))
        .route("/v1/files/:file_id", get(get_files))
        .route("/v1/path", get(get_path))
        .with_state(injector);

    if is_running_on_lambda() {
        // Run app on AWS Lambda
        run(app).await?;
        // run_hyper_on_lambda(app).await?;
    } else {
        // Run app on local server
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
    Ok(())
}
