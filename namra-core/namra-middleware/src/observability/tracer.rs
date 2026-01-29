//! OpenTelemetry tracer initialization

use anyhow::{Context, Result};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, TracerProvider};
use opentelemetry_sdk::Resource;
use serde::{Deserialize, Serialize};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Observability configuration for OpenTelemetry tracing
///
/// Supported exporters:
/// - `jaeger`: Exports to Jaeger using OTLP gRPC (native support, no translation)
///   Default endpoint: http://localhost:4317 (gRPC)
/// - `otlp`: Generic OTLP exporter using gRPC
///   Default endpoint: http://localhost:4317 (gRPC)
/// - `phoenix`: Exports to Arize Phoenix using OTLP HTTP (LLM-specific observability)
///   Default endpoint: http://localhost:6006 (HTTP, /v1/traces added automatically)
/// - `otlp-http`: Generic OTLP exporter using HTTP
///   Default endpoint: http://localhost:4318 (HTTP, /v1/traces added automatically)
/// - `stdout`: Prints spans to console (for debugging)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub enabled: bool,
    pub trace_all_steps: bool,
    pub export_to: Option<String>,
    pub endpoint: Option<String>,
    pub sample_rate: f32,
    pub metrics: Vec<String>,
    /// Enable capture of LLM prompt/response and tool input/output content
    pub capture_content: bool,
    /// Maximum content size in bytes (default: 4000)
    pub max_content_size: usize,
}

pub struct NamraTracer {
    _provider: TracerProvider,
}

impl NamraTracer {
    /// Initialize OpenTelemetry tracer
    pub fn init(config: &ObservabilityConfig) -> Result<Self> {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        if !config.enabled {
            // Initialize basic tracing without OpenTelemetry
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer())
                .try_init()
                .context("Failed to initialize tracing")?;

            return Ok(Self {
                _provider: TracerProvider::builder().build(),
            });
        }

        // Get exporter from config or env
        let export_to = config
            .export_to
            .clone()
            .or_else(|| std::env::var("NAMRA_OTEL_EXPORTER").ok())
            .unwrap_or_else(|| "stdout".to_string());

        // Get endpoint with smart defaults based on exporter type
        let endpoint = config
            .endpoint
            .clone()
            .or_else(|| std::env::var("NAMRA_OTEL_ENDPOINT").ok())
            .unwrap_or_else(|| {
                // Default endpoints for different exporters
                // Note: HTTP exporters append /v1/traces automatically, so we provide base URL only
                match export_to.as_str() {
                    "jaeger" => "http://localhost:4317".to_string(), // Jaeger OTLP gRPC
                    "otlp" => "http://localhost:4317".to_string(),   // OTLP gRPC
                    "phoenix" => "http://localhost:6006".to_string(), // Phoenix OTLP HTTP (base URL)
                    "otlp-http" => "http://localhost:4318".to_string(), // OTLP HTTP (base URL)
                    _ => "http://localhost:4317".to_string(),
                }
            });

        // Create resource with service info
        let resource = Resource::new(vec![
            KeyValue::new("service.name", "namra"),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ]);

        // Create the tracer provider
        // Note: "jaeger" uses OTLP since Jaeger natively supports OTLP (no translation needed)
        // "phoenix" and "otlp-http" use HTTP protocol for better compatibility
        let provider = match export_to.as_ref() {
            "jaeger" | "otlp" => {
                create_otlp_grpc_provider(&endpoint, config.sample_rate, resource)?
            }
            "phoenix" | "otlp-http" => {
                create_otlp_http_provider(&endpoint, config.sample_rate, resource)?
            }
            "stdout" => create_stdout_provider(config.sample_rate, resource)?,
            _ => anyhow::bail!(
                "Unknown exporter type: {}. Use: jaeger, otlp, phoenix, otlp-http, or stdout",
                export_to
            ),
        };

        // Create tracing subscriber with OpenTelemetry layer
        let tracer = provider.tracer("namra");
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        // Set global tracer provider
        global::set_tracer_provider(provider.clone());

        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .with(telemetry)
            .try_init()
            .context("Failed to initialize tracing")?;

        Ok(Self {
            _provider: provider,
        })
    }

    /// Shutdown the tracer (flush pending spans)
    pub fn shutdown(self) {
        global::shutdown_tracer_provider();
    }
}

fn create_otlp_grpc_provider(
    endpoint: &str,
    sample_rate: f32,
    resource: Resource,
) -> Result<TracerProvider> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint)
        .build_span_exporter()
        .context("Failed to create OTLP gRPC exporter")?;

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_config(
            opentelemetry_sdk::trace::Config::default()
                .with_sampler(Sampler::TraceIdRatioBased(sample_rate as f64))
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(resource),
        )
        .build();

    Ok(provider)
}

fn create_otlp_http_provider(
    endpoint: &str,
    sample_rate: f32,
    resource: Resource,
) -> Result<TracerProvider> {
    let exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint(endpoint)
        .build_span_exporter()
        .context("Failed to create OTLP HTTP exporter")?;

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_config(
            opentelemetry_sdk::trace::Config::default()
                .with_sampler(Sampler::TraceIdRatioBased(sample_rate as f64))
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(resource),
        )
        .build();

    Ok(provider)
}

fn create_stdout_provider(sample_rate: f32, resource: Resource) -> Result<TracerProvider> {
    let exporter = opentelemetry_stdout::SpanExporter::default();

    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_config(
            opentelemetry_sdk::trace::Config::default()
                .with_sampler(Sampler::TraceIdRatioBased(sample_rate as f64))
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(resource),
        )
        .build();

    Ok(provider)
}
