use anyhow::{Ok, Result};
use axum::routing::get;
use axum::Router;
use once_cell::sync::Lazy;
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, WithExportConfig};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::{RandomIdGenerator, TracerProvider};
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

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![
        KeyValue::new(resource::SERVICE_NAME, "axum-tracing-example"),
        KeyValue::new(resource::SERVICE_VERSION, "0.0.1"),
        KeyValue::new(resource::SERVICE_NAMESPACE, "jrmarcco"),
    ])
});

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

    let tracer_provider = init_tracer_provider()?;
    global::set_tracer_provider(tracer_provider.clone());

    let metrics_provider = init_metrics()?;
    global::set_meter_provider(metrics_provider.clone());

    let logger_provider = init_logs()?;
    let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    let _ = tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(logger_layer)
        .try_init();

    let addr = "0.0.0.0:8080";
    let app = Router::new().route("/", get(index_handler));

    let listener = TcpListener::bind(addr).await?;
    info!("Start server on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

fn init_tracer_provider() -> Result<TracerProvider> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    Ok(trace::TracerProvider::builder()
        .with_resource(RESOURCE.clone())
        .with_id_generator(RandomIdGenerator::default())
        .with_batch_exporter(exporter, runtime::Tokio)
        .build())
}

fn init_metrics() -> Result<SdkMeterProvider> {
    let exporter = MetricExporter::builder().with_tonic().build()?;

    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();

    Ok(SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(RESOURCE.clone())
        .build())
}

fn init_logs() -> Result<LoggerProvider> {
    let exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    Ok(LoggerProvider::builder()
        .with_resource(RESOURCE.clone())
        .with_batch_exporter(exporter, runtime::Tokio)
        .build())
}

#[instrument]
async fn index_handler() -> &'static str {
    "Hello, World"
}
