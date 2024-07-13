use anyhow::Result;
use async_injector::{Injector, Provider};
use axum::{extract::State, Router, routing::get};
use axum::{
    extract::{Path, Query},
    Json,
    routing::post,
};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres};
use sqlx::migrate::Migrator;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use uuid::Uuid;

const TABLE: &str = "rust-test";

#[derive(Serialize, FromRow)]
struct RustTest {
    uuid: Uuid,
    hash: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Provider)]
struct Repository {
    #[dependency]
    client: sqlx::Pool<Postgres>,
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
    file_id: Uuid,
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
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = Repository::provider(&injector)
        .await
        .unwrap()
        .wait()
        .await
        .client;

    let query = format!(
        r#"
        INSERT INTO {} (uuid, hash)
        VALUES ($1, $2)
        "#,
        TABLE
    );
    sqlx::query(&query)
        .bind(file_id)
        .bind(payload.hash)
        .execute(&client)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Json(UploadResponse {
        upload_url: "https://example.com/upload/42".to_string(),
    }))
}

async fn get_analysis(
    State(injector): State<&'static Injector>,
    Path(file_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let _client = Repository::provider(&injector)
        .await
        .unwrap()
        .wait()
        .await
        .client;

    Ok(Json(AnalysisResponse {
        status: "processing".to_string(),
        status_message: "".to_string(),
        result_url: format!("https://ll09yudnr6.execute-api.us-east-1.amazonaws.com/v1/results/{file_id}"),
    }))
}

async fn get_results(
    State(injector): State<&'static Injector>,
    Path(_file_id): Path<Uuid>,
    pagination: Query<PaginationQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = Repository::provider(&injector)
        .await
        .unwrap()
        .wait()
        .await
        .client;

    let page_size = pagination.per_page;
    let items: Vec<RustTest> = sqlx::query_as(r#"SELECT uuid, created_at, hash
        FROM rust_test
        LIMIT $1"#)
        .bind(page_size as i64)
        .fetch_all(&client)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let ids = items.iter()
        .map(|item| item.uuid)
        .collect::<Vec<_>>();

    Ok(Json(AnalysisResultResponse {
        status: "processed".to_string(),
        status_message: "".to_string(),
        related_file_ids: ids,
    }))
}

async fn get_files(
    State(injector): State<&'static Injector>,
    Path(_file_id): Path<Uuid>,
    _after_date: Query<AfterDateQuery>,
    pagination: Query<PaginationQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = Repository::provider(&injector)
        .await
        .unwrap()
        .wait()
        .await
        .client;

    let page_size = pagination.per_page;
    let items: Vec<RustTest> = sqlx::query_as!(
        RustTest,
        r#"
        SELECT uuid, created_at, hash
        FROM rust_test
        LIMIT $1
        "#,
        page_size as i32
    )
        .fetch_all(&client)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let files = items.iter().map(|item| {
        let uuid = item.uuid;
        let created_at = item.created_at;
        let hash = item.hash.clone();
        File {
            file_id: uuid,
            upload_date: created_at,
            hash,
        }
    }).collect::<Vec<_>>();

    Ok(Json(FilesResponse { files }))
}

async fn get_path(
    State(injector): State<&'static Injector>,
    ids: Query<IdsSrcDstQuery>,
    _pagination: Query<PaginationQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = Repository::provider(&injector)
        .await
        .unwrap()
        .wait()
        .await
        .client;

    let ids_src: Uuid = ids.src;
    let ids_dst: Uuid = ids.dst;

    let ids: Vec<Uuid> = sqlx::query!(
        r#"
        SELECT uuid
        FROM rust_test
        WHERE uuid IN ($1, $2)
        "#,
        ids_src,
        ids_dst
    )
        .fetch_all(&client)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .iter()
        .map(|row| row.uuid)
        .collect();

    Ok(Json(PathResponse {
        path: ids,
    }))
}

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[shuttle_runtime::main]
async fn axum(#[shuttle_shared_db::Postgres] database_url: String) -> shuttle_axum::ShuttleAxum {
    // Create the database connection pool
    let pool = PgPool::connect(&database_url).await
        .map_err(|err| {
            println!("Failed to create pool: {:?}", err);
            shuttle_runtime::Error::Custom(err.into())
        })?;

    // Run migrations
    MIGRATOR.run(&pool).await
        .map_err(|err| {
            println!("Failed to run migrations: {:?}", err);
            shuttle_runtime::Error::Custom(err.into())
        })?;

    let client = sqlx::Pool::<Postgres>::connect(&database_url).await.unwrap();

    let injector = Injector::new();
    injector.update(client).await;

    // let init_provider = InitProvider(provider);
    let injector: &'static Injector = Box::leak(Box::new(injector));

    println!("Starting server...");

    // build our application with a route
    let app = Router::new()
        .route("/v1/upload/:file_id", post(post_upload))
        .route("/v1/analysis/:file_id", get(get_analysis))
        .route("/v1/results/:file_id", get(get_results))
        .route("/v1/files/:file_id", get(get_files))
        .route("/v1/path", get(get_path))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
        )
        .with_state(injector);

    Ok(app.into())
}
