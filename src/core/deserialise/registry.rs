//! Registry builder that creates a configured registry from metric definitions.

#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use crate::core::metrics::Metric;
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use crate::core::registry::{MetricBackend, ObservabilityRegistry};
use crate::core::deserialise::errors::{DeserializeError, BackendErrorExt};
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use crate::core::deserialise::config::RegistryConfig;
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use std::collections::HashMap;

/// A registry that has been configured from a config file, with metrics
/// accessible by name.
///
/// This structure provides both the registry for rendering metrics and
/// HashMaps to access individual metrics by their name for runtime updates.
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
pub struct ConfiguredRegistry<B: MetricBackend> {
    /// The underlying metrics registry.
    pub registry: ObservabilityRegistry<B>,
    /// All counter metrics, indexed by their title/name.
    pub counters: HashMap<String, Metric<B::Counter>>,
    /// All gauge metrics, indexed by their title/name.
    pub gauges: HashMap<String, Metric<B::Gauge>>,
    /// All histogram metrics, indexed by their title/name.
    pub histograms: HashMap<String, Metric<B::Histogram>>,
}

#[cfg(any(feature = "json-config", feature = "yaml-config"))]
impl<B: MetricBackend> ConfiguredRegistry<B> {
    /// Create a configured registry from a `RegistryConfig`.
    ///
    /// This performs optimized deserialization by:
    /// - Pre-counting metrics to allocate HashMaps with appropriate capacity
    /// - Single-pass registration of all metrics
    /// - Minimal allocations and cloning
    ///
    /// # Example
    /// ```ignore
    /// use observability_kit::core::deserialise::{RegistryConfig, ConfiguredRegistry};
    /// use observability_kit::backends::prometheus::PrometheusBackend;
    ///
    /// let config: RegistryConfig = vec![/* ... */];
    /// let configured = ConfiguredRegistry::<PrometheusBackend>::from_config(config)?;
    ///
    /// // Access metrics by name
    /// let requests = configured.counters.get("http_requests_total").unwrap();
    /// requests.inc();
    /// ```
    pub fn from_config(config: RegistryConfig) -> Result<Self, DeserializeError> {
        let metrics = config;

        // Pre-count metrics by type to allocate HashMaps with appropriate capacity
        // This avoids rehashing as the HashMaps grow
        let (counter_count, gauge_count, histogram_count) = metrics.iter().fold(
            (0, 0, 0),
            |(c, g, h), m| match m {
                crate::core::deserialise::config::MetricConfig::Counter { .. } => (c + 1, g, h),
                crate::core::deserialise::config::MetricConfig::Gauge { .. } => (c, g + 1, h),
                crate::core::deserialise::config::MetricConfig::Histogram { .. } => (c, g, h + 1),
            },
        );

        let mut registry = ObservabilityRegistry::<B>::new();
        let mut counters = HashMap::with_capacity(counter_count);
        let mut gauges = HashMap::with_capacity(gauge_count);
        let mut histograms = HashMap::with_capacity(histogram_count);

        // Single-pass registration: deserialize and register all metrics
        for metric_config in metrics {
            match metric_config {
                crate::core::deserialise::config::MetricConfig::Counter {
                    title,
                    description,
                    initial_value,
                } => {
                    let counter = registry
                        .counter(&title, &description)
                        .map_err(|e| e.into_deserialize_error())?;
                    if initial_value > 0 {
                        counter.inc_by(initial_value);
                    }
                    counters.insert(title, counter);
                }
                crate::core::deserialise::config::MetricConfig::Gauge {
                    title,
                    description,
                    initial_value,
                } => {
                    let gauge = registry
                        .gauge(&title, &description)
                        .map_err(|e| e.into_deserialize_error())?;
                    if initial_value != 0 {
                        gauge.set(initial_value);
                    }
                    gauges.insert(title, gauge);
                }
                crate::core::deserialise::config::MetricConfig::Histogram {
                    title,
                    description,
                    buckets,
                } => {
                    let histogram = registry
                        .histogram_with_buckets(&title, &description, buckets)
                        .map_err(|e| e.into_deserialize_error())?;
                    histograms.insert(title, histogram);
                }
            }
        }

        Ok(Self {
            registry,
            counters,
            gauges,
            histograms,
        })
    }

    /// Create a configured registry without storing metric references.
    ///
    /// This is faster if you don't need to access metrics by name after creation.
    /// It simply registers all metrics and returns the registry.
    ///
    /// # Example
    /// ```ignore
    /// let config: RegistryConfig = vec![/* ... */];
    /// let registry = ConfiguredRegistry::<PrometheusBackend>::registry_only(config)?;
    /// let output = registry.render()?;
    /// ```
    pub fn registry_only(config: RegistryConfig) -> Result<ObservabilityRegistry<B>, DeserializeError> {
        let metrics = config;
        let mut registry = ObservabilityRegistry::<B>::new();

        // Single-pass registration without storing references
        for metric_config in metrics {
            match metric_config {
                crate::core::deserialise::config::MetricConfig::Counter {
                    title,
                    description,
                    initial_value,
                } => {
                    let counter = registry
                        .counter(&title, &description)
                        .map_err(|e| e.into_deserialize_error())?;
                    if initial_value > 0 {
                        counter.inc_by(initial_value);
                    }
                }
                crate::core::deserialise::config::MetricConfig::Gauge {
                    title,
                    description,
                    initial_value,
                } => {
                    let gauge = registry
                        .gauge(&title, &description)
                        .map_err(|e| e.into_deserialize_error())?;
                    if initial_value != 0 {
                        gauge.set(initial_value);
                    }
                }
                crate::core::deserialise::config::MetricConfig::Histogram {
                    title,
                    description,
                    buckets,
                } => {
                    registry
                        .histogram_with_buckets(&title, &description, buckets)
                        .map_err(|e| e.into_deserialize_error())?;
                }
            }
        }

        Ok(registry)
    }
}
