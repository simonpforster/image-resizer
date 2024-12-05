use opentelemetry::{Context, KeyValue};
use opentelemetry::trace::{Span as _, TraceResult, TracerProvider as _};
use opentelemetry_sdk::export::trace::SpanData;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::{Sampler, Span, SpanProcessor, TracerProvider};
use tracing::metadata::LevelFilter;
use tracing_stackdriver::CloudTraceConfiguration;
use tracing_subscriber::filter::ParseError;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub async fn init_tracing() -> Result<(), ParseError> {

    let authorizer = opentelemetry_stackdriver::GcpAuthorizer::new()
        .await
        .expect("Failed to create GCP authorizer.");

    let (stackdriver_tracer, driver) = opentelemetry_stackdriver::Builder::default()
        .build(authorizer)
        .await
        .expect("Failed to create Stackdriver tracer.");

    tokio::spawn(driver);

    let provider = TracerProvider::builder()
        .with_batch_exporter(stackdriver_tracer, Tokio)
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(1.0))))
        .with_span_processor(CustomSpanProcessor::new())
        .build();

    let tracer = provider.tracer("image-resizer");

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let stackdriver_layer =
        tracing_stackdriver::layer().with_cloud_trace(CloudTraceConfiguration {
            project_id: "listen-and-learn-411214".to_string(),
        });


    let subscriber = Registry::default()
        .with(EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy())
        .with(otel_layer)
        .with(stackdriver_layer);

    opentelemetry::global::set_tracer_provider(provider);
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
    tracing::subscriber::set_global_default(subscriber).expect("No subscriber set!!!");
    Ok(())
}


#[derive(Debug)]
pub struct CustomSpanProcessor {}

impl CustomSpanProcessor {
    pub fn new() -> Self {
        CustomSpanProcessor {}
    }
}

const GCP_SERVICE_NAME_ATTRIBUTE: &str = "service.name";

const SERVICE_NAME: &str = "image-resizer";

impl SpanProcessor for CustomSpanProcessor {
    fn on_start(&self, span: &mut Span, _cx: &Context) {
        span.set_attribute(KeyValue::new(
            GCP_SERVICE_NAME_ATTRIBUTE,
            SERVICE_NAME,
        ));
    }

    fn on_end(&self, _span: SpanData) {}

    fn force_flush(&self) -> TraceResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> TraceResult<()> {
        Ok(())
    }
}