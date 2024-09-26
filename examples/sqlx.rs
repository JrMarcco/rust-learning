use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use http::{header::LOCATION, HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

const LISTEN_ADDR: &str = "127.0.0.1:3000";
const DB_URL: &str = "postgres://jrmarcco:postgres@localhost:5432/test_db";

#[derive(Debug, Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Debug, Deserialize)]
struct ShortenReq {
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenRes {
    url: String,
}

#[derive(Debug, FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let state = AppState::try_new(DB_URL).await?;
    info!("connected to db: {}", DB_URL);

    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    info!("listening on {}", LISTEN_ADDR);

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn shorten(
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = state.shorten(&data.url).await.map_err(|e| {
        warn!("failed to shorten url: {}", e);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let body = Json(ShortenRes {
        url: format!("http://{}/{}", LISTEN_ADDR, id),
    });

    Ok((StatusCode::CREATED, body))
}

async fn redirect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let url = state.get_url(&id).await.map_err(|e| {
        warn!("failed to get url: {}", e);
        StatusCode::NOT_FOUND
    })?;

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());

    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}

impl AppState {
    async fn try_new(db_url: &str) -> Result<Self> {
        let db = PgPool::connect(db_url).await?;
        Ok(Self { db })
    }

    async fn shorten(&self, url: &str) -> Result<String> {
        let id = nanoid::nanoid!(6);

        let ret: UrlRecord = sqlx::query_as(
            "INSERT INTO t_url (id, url) VALUES ($1, $2) ON CONFLICT (url) DO UPDATE SET url = EXCLUDED.url RETURNING id",
        )
        .bind(id)
        .bind(url)
        .fetch_one(&self.db)
        .await?;

        Ok(ret.id)
    }

    async fn get_url(&self, id: &str) -> Result<String> {
        let ret: UrlRecord = sqlx::query_as("SELECT id, url FROM t_url WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;

        Ok(ret.url)
    }
}
