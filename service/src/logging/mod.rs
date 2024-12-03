use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::TracerProvider;
use tracing::metadata::LevelFilter;
use tracing_subscriber::filter::ParseError;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn init_tracing() -> Result<(), ParseError> {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let tracer = provider.tracer("image-resizer");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = Registry::default()
        .with(EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy())
        .with(telemetry);
    tracing::subscriber::set_global_default(subscriber).expect("No subscriber set!!!");
    Ok(())
}