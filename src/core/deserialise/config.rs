//! Configuration structures for deserializing metric definitions.
//!
//! These types work for both JSON and YAML deserialization since they use
//! the same serde Deserialize trait. The configuration format is format-agnostic.

#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use serde::Deserialize;

/// Configuration for a single metric definition.
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "metric_type", rename_all = "PascalCase")]
pub enum MetricConfig {
    Counter {
        title: String,
        description: String,
        #[serde(default, rename = "value")]
        initial_value: u64,
    },
    Gauge {
        title: String,
        description: String,
        #[serde(default, rename = "value")]
        initial_value: i64,
    },
    Histogram {
        title: String,
        description: String,
        #[serde(default = "default_histogram_buckets")]
        buckets: Vec<f64>,
    },
}

fn default_histogram_buckets() -> Vec<f64> {
    crate::core::registry::DEFAULT_LATENCY_BUCKETS.to_vec()
}

/// Complete registry configuration - a vector of metric definitions.
///
/// This deserializes directly from an array format: `[{...}, {...}]`
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
pub type RegistryConfig = Vec<MetricConfig>;
