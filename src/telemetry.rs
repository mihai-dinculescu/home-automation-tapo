use std::error::Error;
use std::sync::OnceLock;

use opentelemetry::trace::TracerProvider;
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use opentelemetry_semantic_conventions as semconv;
use tracing::{Level, Span};
use tracing_subscriber::Registry;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _};

use crate::settings::Telemetry;

pub fn record_error(span: &Span, e: &impl Error) {
    span.record(semconv::attribute::OTEL_STATUS_CODE, "ERROR");
    span.record(
        semconv::attribute::EXCEPTION_TYPE,
        "SendError<DeviceUsageMessage>",
    );
    span.record(semconv::attribute::EXCEPTION_MESSAGE, e.to_string());
    span.record(semconv::attribute::EXCEPTION_STACKTRACE, format!("{e:?}"));
}

pub fn init_telemetry(settings: &Telemetry) -> Result<SdkTracerProvider, Box<dyn Error>> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(settings.otlp_endpoint.clone())
        .build()
        .expect("Failed to create span exporter");

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(get_resource(settings))
        .with_batch_exporter(exporter)
        .build();

    let tracer = tracer_provider.tracer(settings.service_name.clone());

    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(Level::INFO.to_string()))
        // .add_directive("reqwest=off".parse()?)
        ;

    let tracing_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let format_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_line_number(true)
        // .with_span_events(FmtSpan::ACTIVE)
        ;

    let subscriber = Registry::default()
        .with(filter_layer)
        .with(tracing_layer)
        .with(format_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(tracer_provider)
}

pub fn shutdown_telemetry(tracer_provider: SdkTracerProvider) -> Result<(), Box<dyn Error>> {
    // Collect all shutdown errors
    let mut shutdown_errors = Vec::new();

    if let Err(e) = tracer_provider.shutdown() {
        shutdown_errors.push(format!("tracer provider: {e}"));
    }

    // Return an error if any shutdown failed
    if !shutdown_errors.is_empty() {
        return Err(format!(
            "Failed to shutdown providers:{}",
            shutdown_errors.join("\n")
        )
        .into());
    }

    Ok(())
}

fn get_resource(settings: &Telemetry) -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_attributes(vec![
                    KeyValue::new(
                        semconv::attribute::SERVICE_NAME,
                        settings.service_name.clone(),
                    ),
                    KeyValue::new(
                        semconv::attribute::SERVICE_NAMESPACE,
                        settings.service_namespace.clone(),
                    ),
                    KeyValue::new(
                        semconv::attribute::DEPLOYMENT_ENVIRONMENT_NAME,
                        settings.deployment_environment.clone(),
                    ),
                    KeyValue::new(
                        semconv::attribute::SERVICE_VERSION,
                        env!("CARGO_PKG_VERSION"),
                    ),
                ])
                .build()
        })
        .clone()
}
