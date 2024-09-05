use anyhow::Result;
use axum::routing::get;
use axum::Router;
use opentelemetry::trace::TracerProvider;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{RandomIdGenerator, Tracer};
use opentelemetry_sdk::{runtime, trace, Resource};
use opentelemetry_semantic_conventions::resource;
use tokio::net::TcpListener;
use tracing::{info, instrument, level_filters::LevelFilter};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    Layer,
};

#[tokio::main]
async fn main() -> Result<()> {
    let console_layer = fmt::Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::INFO);

    let file_appender = tracing_appender::rolling::daily("./logs", "rust-learning.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::Layer::new()
        .with_writer(non_blocking)
        .with_filter(LevelFilter::INFO);

    let tracer = init_tracer()?;
    let oltp = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(oltp)
        .init();

    let addr = "0.0.0.0:8080";
    let app = Router::new().route("/", get(index_handler));

    let listener = TcpListener::bind(addr).await?;
    info!("Start server on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[instrument]
async fn index_handler() -> &'static str {
    "Hello, World"
}

fn init_tracer() -> Result<Tracer> {
    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            trace::Config::default()
                .with_resource(Resource::new(vec![
                    KeyValue::new(resource::SERVICE_NAME, "axum-tracing-example"),
                    KeyValue::new(resource::SERVICE_VERSION, "0.0.1"),
                    KeyValue::new(resource::SERVICE_NAMESPACE, "jrmarcco"),
                ]))
                .with_id_generator(RandomIdGenerator::default()),
        )
        .install_batch(runtime::Tokio)?;

    global::set_tracer_provider(tracer_provider.clone());

    let tracer = tracer_provider.tracer("axum-tracing-example");
    Ok(tracer)
}
