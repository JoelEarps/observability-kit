//! Registry builder that creates a configured registry from metric definitions.

#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use crate::core::deserialise::config::RegistryConfig;
use crate::core::deserialise::errors::{BackendErrorExt, DeserializeError};
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use crate::core::metrics::Metric;
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use crate::core::registry::{MetricBackend, ObservabilityRegistry};
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use std::collections::hash_map::Entry;
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

/// If `title` is already in `map`, returns `DuplicateMetricName`. Otherwise calls `make(key)`,
/// inserts the result, and returns `Ok(())`.
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
fn register_unique_metric<V>(
    map: &mut HashMap<String, V>,
    title: String,
    make: impl FnOnce(&str) -> Result<V, DeserializeError>,
) -> Result<(), DeserializeError> {
    match map.entry(title) {
        Entry::Occupied(o) => Err(DeserializeError::DuplicateMetricName(o.key().clone())),
        Entry::Vacant(e) => {
            let value = make(e.key().as_str())?;
            e.insert(value);
            Ok(())
        }
    }
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
        let (counter_count, gauge_count, histogram_count) =
            metrics.iter().fold((0, 0, 0), |(c, g, h), m| match m {
                crate::core::deserialise::config::MetricConfig::Counter { .. } => (c + 1, g, h),
                crate::core::deserialise::config::MetricConfig::Gauge { .. } => (c, g + 1, h),
                crate::core::deserialise::config::MetricConfig::Histogram { .. } => (c, g, h + 1),
            });

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
                    register_unique_metric(&mut counters, title, |key| {
                        let counter = registry
                            .counter(key, &description)
                            .map_err(|e| e.into_deserialize_error())?;
                        if initial_value > 0 {
                            counter.inc_by(initial_value);
                        }
                        Ok(counter)
                    })?;
                }
                crate::core::deserialise::config::MetricConfig::Gauge {
                    title,
                    description,
                    initial_value,
                } => {
                    register_unique_metric(&mut gauges, title, |key| {
                        let gauge = registry
                            .gauge(key, &description)
                            .map_err(|e| e.into_deserialize_error())?;
                        if initial_value != 0 {
                            gauge.set(initial_value);
                        }
                        Ok(gauge)
                    })?;
                }
                crate::core::deserialise::config::MetricConfig::Histogram {
                    title,
                    description,
                    buckets,
                } => {
                    register_unique_metric(&mut histograms, title, |key| {
                        registry
                            .histogram_with_buckets(key, &description, buckets)
                            .map_err(|e| e.into_deserialize_error())
                    })?;
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
    pub fn registry_only(
        config: RegistryConfig,
    ) -> Result<ObservabilityRegistry<B>, DeserializeError> {
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

#[cfg(test)]
#[cfg(all(feature = "json-config", feature = "prometheus"))]
mod tests {
    use super::*;
    use crate::backends::prometheus::prometheus_backend::PrometheusBackend;
    use crate::core::deserialise::config::MetricConfig;

    fn counter_config(title: &str, description: &str, initial_value: u64) -> MetricConfig {
        MetricConfig::Counter {
            title: title.to_string(),
            description: description.to_string(),
            initial_value,
        }
    }

    fn gauge_config(title: &str, description: &str, initial_value: i64) -> MetricConfig {
        MetricConfig::Gauge {
            title: title.to_string(),
            description: description.to_string(),
            initial_value,
        }
    }

    fn histogram_config(title: &str, description: &str, buckets: Vec<f64>) -> MetricConfig {
        MetricConfig::Histogram {
            title: title.to_string(),
            description: description.to_string(),
            buckets,
        }
    }

    #[test]
    fn from_config_empty_returns_ok_with_empty_maps() {
        let config: RegistryConfig = vec![];
        let result = ConfiguredRegistry::<PrometheusBackend>::from_config(config);
        assert!(result.is_ok());
        let configured = result.unwrap();
        assert!(configured.counters.is_empty());
        assert!(configured.gauges.is_empty());
        assert!(configured.histograms.is_empty());
    }

    #[test]
    fn from_config_single_counter_accessible_by_name() {
        let config: RegistryConfig = vec![counter_config("requests_total", "Total requests", 0)];
        let configured = ConfiguredRegistry::<PrometheusBackend>::from_config(config).unwrap();
        assert_eq!(configured.counters.len(), 1);
        let c = configured.counters.get("requests_total").unwrap();
        assert_eq!(c.get_counter(), 0);
        c.inc();
        assert_eq!(c.get_counter(), 1);
    }

    #[test]
    fn from_config_counter_initial_value_set() {
        let config: RegistryConfig = vec![counter_config("count", "A counter", 10)];
        let configured = ConfiguredRegistry::<PrometheusBackend>::from_config(config).unwrap();
        let c = configured.counters.get("count").unwrap();
        assert_eq!(c.get_counter(), 10);
    }

    #[test]
    fn from_config_single_gauge_accessible_by_name() {
        let config: RegistryConfig = vec![gauge_config("active", "Active connections", 0)];
        let configured = ConfiguredRegistry::<PrometheusBackend>::from_config(config).unwrap();
        assert_eq!(configured.gauges.len(), 1);
        let g = configured.gauges.get("active").unwrap();
        g.set(42);
        assert_eq!(g.get_gauge(), 42);
    }

    #[test]
    fn from_config_gauge_initial_value_set() {
        let config: RegistryConfig = vec![gauge_config("level", "Level", 100)];
        let configured = ConfiguredRegistry::<PrometheusBackend>::from_config(config).unwrap();
        let g = configured.gauges.get("level").unwrap();
        assert_eq!(g.get_gauge(), 100);
    }

    #[test]
    fn from_config_single_histogram_accessible_by_name() {
        let buckets = vec![0.1, 0.5, 1.0];
        let config: RegistryConfig = vec![histogram_config("latency", "Latency", buckets)];
        let configured = ConfiguredRegistry::<PrometheusBackend>::from_config(config).unwrap();
        assert_eq!(configured.histograms.len(), 1);
        let h = configured.histograms.get("latency").unwrap();
        h.observe(0.25);
    }

    #[test]
    fn from_config_duplicate_counter_name_returns_error() {
        let config: RegistryConfig = vec![
            counter_config("same_name", "First", 0),
            counter_config("same_name", "Second", 0),
        ];
        let result = ConfiguredRegistry::<PrometheusBackend>::from_config(config);
        match result {
            Err(DeserializeError::DuplicateMetricName(name)) => assert_eq!(name, "same_name"),
            Err(e) => panic!("expected DuplicateMetricName, got error: {:?}", e),
            Ok(_) => panic!("expected DuplicateMetricName, got Ok"),
        }
    }

    #[test]
    fn from_config_duplicate_gauge_name_returns_error() {
        let config: RegistryConfig = vec![
            gauge_config("same_gauge", "First", 0),
            gauge_config("same_gauge", "Second", 0),
        ];
        let result = ConfiguredRegistry::<PrometheusBackend>::from_config(config);
        match result {
            Err(DeserializeError::DuplicateMetricName(name)) => assert_eq!(name, "same_gauge"),
            Err(e) => panic!("expected DuplicateMetricName, got error: {:?}", e),
            Ok(_) => panic!("expected DuplicateMetricName, got Ok"),
        }
    }

    #[test]
    fn from_config_duplicate_histogram_name_returns_error() {
        let buckets = vec![1.0, 2.0];
        let config: RegistryConfig = vec![
            histogram_config("same_hist", "First", buckets.clone()),
            histogram_config("same_hist", "Second", buckets),
        ];
        let result = ConfiguredRegistry::<PrometheusBackend>::from_config(config);
        match result {
            Err(DeserializeError::DuplicateMetricName(name)) => assert_eq!(name, "same_hist"),
            Err(e) => panic!("expected DuplicateMetricName, got error: {:?}", e),
            Ok(_) => panic!("expected DuplicateMetricName, got Ok"),
        }
    }

    #[test]
    fn from_config_same_name_different_types_allowed() {
        let config: RegistryConfig = vec![
            counter_config("metric", "Counter", 0),
            gauge_config("metric", "Gauge", 0),
            histogram_config("metric", "Histogram", vec![1.0]),
        ];
        let result = ConfiguredRegistry::<PrometheusBackend>::from_config(config);
        assert!(result.is_ok());
        let configured = result.unwrap();
        assert!(configured.counters.contains_key("metric"));
        assert!(configured.gauges.contains_key("metric"));
        assert!(configured.histograms.contains_key("metric"));
    }

    #[test]
    fn registry_only_registers_without_storing_refs() {
        let config: RegistryConfig = vec![
            counter_config("c", "Counter", 0),
            gauge_config("g", "Gauge", 0),
        ];
        let result = ConfiguredRegistry::<PrometheusBackend>::registry_only(config);
        assert!(result.is_ok());
        let registry = result.unwrap();
        let output = registry.render();
        assert!(output.is_ok());
    }
}
