use anyhow::Result;
use axum::routing::patch;
use axum::{
    extract::State,
    routing::get,
    {Json, Router},
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tracing::{info, instrument, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer as _,
};

#[derive(Debug, Clone, Serialize)]
struct Foo {
    name: String,
    number: u8,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FooUpdate {
    name: Option<String>,
    tags: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry().with(console).init();

    let state = Foo {
        name: "bar".to_string(),
        number: 10,
        tags: vec!["Rust".to_string(), "Golang".to_string()],
    };

    let state = Arc::new(Mutex::new(state));

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {}", addr);

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/", patch(update_handler))
        .with_state(state);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[instrument]
async fn index_handler(State(state): State<Arc<Mutex<Foo>>>) -> Json<Foo> {
    (*state.lock().unwrap()).clone().into()
}

#[instrument]
async fn update_handler(
    State(state): State<Arc<Mutex<Foo>>>,
    Json(state_update): Json<FooUpdate>,
) -> Json<Foo> {
    let mut instance = state.lock().unwrap();

    if let Some(name) = state_update.name {
        instance.name = name;
    }

    if let Some(tags) = state_update.tags {
        instance.tags = tags;
    }

    (*instance).clone().into()
}
