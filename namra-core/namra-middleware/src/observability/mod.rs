//! Observability module for OpenTelemetry tracing and metrics

pub mod spans;
pub mod tracer;

pub use spans::*;
pub use tracer::{NamraTracer, ObservabilityConfig};
