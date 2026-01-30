# Observability Kit

A flexible, multi-backend observability library for Rust applications.

[![Crates.io](https://img.shields.io/crates/v/observability-kit.svg)](https://crates.io/crates/observability-kit)
[![Documentation](https://docs.rs/observability-kit/badge.svg)](https://docs.rs/observability-kit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🔌 **Multi-backend support** — Prometheus (more backends coming soon)
- 🎛️ **Feature-gated** — Only compile what you need
- 🚀 **Standalone server** — Built-in HTTP server for `/metrics`, `/health`, `/ready`
- 🏷️ **Labeled metrics** — Full support for dimensional metrics
- 🧪 **Test utilities** — Mock backend for easy unit testing

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
observability-kit = "0.1"
```

### Standalone Server (Sidecar/Embedded)

Perfect for sidecar deployments or embedded metrics servers:

```rust
use observability_kit::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a standalone server with Prometheus backend
    let server = StandaloneServer::<PrometheusBackend>::builder()
        .port(9090)
        .host("0.0.0.0")
        .build();

    // Create metrics via the registry
    let registry_handle = server.registry();
    let mut registry = registry_handle.write().await;
    
    let requests = registry.counter("http_requests_total", "Total HTTP requests")?;
    let latency = registry.histogram("request_duration_seconds", "Request latency")?;
    
    // Use the metrics
    requests.inc();
    latency.observe(0.042);
    
    drop(registry); // Release the lock before running

    // Start the server
    // Endpoints: /metrics, /health, /ready
    server.run().await?;
    Ok(())
}
```

### Basic Metrics (Without Server)

For simple metric creation without the HTTP server:

```rust
use observability_kit::prelude::*;

// Counters - monotonically increasing values
let requests = counter("http_requests_total", "Total HTTP requests");
requests.inc();
requests.inc_by(5);
println!("Total requests: {}", requests.get_counter()); // 6

// Gauges - values that can go up and down
let connections = gauge("active_connections", "Active connections");
connections.set(42);
connections.inc();
connections.dec();
println!("Connections: {}", connections.get_gauge()); // 42

// Histograms - distributions of values
let latency = histogram_for_latency("request_duration_seconds", "Request latency");
latency.observe(0.042);  // 42ms
latency.observe(0.156);  // 156ms
```

### Labeled Metrics

For dimensional metrics with labels:

```rust
use observability_kit::prelude::*;

// Define your label structure
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct HttpLabels {
    method: String,
    status: u16,
    endpoint: String,
}

// Create a labeled histogram family
let latency: LabeledHistogram<HttpLabels> = labeled_histogram_for_latency();

// Record metrics with specific label values
latency.get_or_create(&HttpLabels {
    method: "GET".into(),
    status: 200,
    endpoint: "/api/users".into(),
}).observe(0.042);

latency.get_or_create(&HttpLabels {
    method: "POST".into(),
    status: 201,
    endpoint: "/api/users".into(),
}).observe(0.156);

// Labeled counters
let requests: LabeledCounter<HttpLabels> = labeled_counter();
requests.get_or_create(&HttpLabels {
    method: "GET".into(),
    status: 200,
    endpoint: "/api/users".into(),
}).inc();
```

### Testing with Mock Backend

The mock backend provides easy testing without a real metrics system:

```rust
use observability_kit::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_counting() {
        let counter = test_counter("test_requests", "For testing");
        
        // Simulate your code that increments the counter
        counter.inc();
        counter.inc_by(5);
        
        // Assert on the value
        assert_eq!(counter.get_counter(), 6);
    }

    #[test]
    fn test_histogram_observations() {
        let histogram = test_histogram("test_latency", "For testing");
        
        histogram.observe(0.1);
        histogram.observe(0.2);
        histogram.observe(0.3);
        
        // Mock histogram tracks all observations
        assert_eq!(histogram.observation_count(), 3);
        assert!((histogram.observation_sum() - 0.6).abs() < 0.001);
    }
}
```

## Histogram Presets

Pre-configured bucket sets for common use cases:

| Function | Buckets | Use Case |
| ---------- | --------- | ---------- |
| `histogram()` | `[0.001, 0.01, 0.1, 1, 10, 100, 1000]` | General purpose |
| `histogram_for_latency()` | `[5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s, 2.5s, 5s, 10s]` | HTTP/API latency |
| `histogram_for_bytes()` | `[100B, 1KB, 10KB, 100KB, 1MB, 10MB, 100MB, 1GB, 10GB, 100GB]` | Response/payload sizes |
| `histogram_with_buckets(buckets)` | Custom | Your own bucket boundaries |

## Feature Flags

| Feature | Description | Default |
| --------- | ------------- | --------- |
| `prometheus` | Prometheus metrics backend | ✅ |
| `standalone` | Standalone HTTP server | ✅ |
| `mock` | Mock backend for testing | |
| `json-config` | JSON configuration support | |
| `yaml-config` | YAML configuration support | |
| `full` | All features | |

### Pick exactly what you need

The crate is fully configurable: use `default-features = false` and list only the features you want. Any combination is supported and tested at publish time.

```toml
# Minimal: Prometheus only (smallest footprint)
observability-kit = { version = "0.1", default-features = false, features = ["prometheus"] }

# Default: Prometheus + standalone server
observability-kit = { version = "0.1" }

# Custom: e.g. Prometheus + JSON config, no server
observability-kit = { version = "0.1", default-features = false, features = ["prometheus", "json-config"] }

# Everything: all backends, config formats, and mock
observability-kit = { version = "0.1", features = ["full"] }
```

## Build Size Comparison

| Feature Combination | Description | Binary Size | Size (KB) | Relative to Minimal |
| --------------------- | ------------- | ------------- | ----------- | --------------------- |
| `prometheus` | Minimal (prometheus only) | 177 KB | 177 KB | 0 KB (baseline) |
| `prometheus,standalone` | Default (prometheus + standalone) | 1.63 MB | 1677 KB | +1500 KB |
| `prometheus,mock` | Prometheus + mock (testing) (lib only) | 229 KB | 229 KB | +52 KB |
| `prometheus,standalone,json-config` | Prometheus + standalone + JSON config | 1.63 MB | 1676 KB | +1499 KB |
| `prometheus,standalone,yaml-config` | Prometheus + standalone + YAML config | 1.63 MB | 1676 KB | +1499 KB |
| `prometheus,standalone,json-config,yaml-config` | Prometheus + standalone + all config formats | 1.63 MB | 1676 KB | +1499 KB |
| `prometheus,standalone,axum-integration` | Prometheus + standalone + Axum integration | 1.63 MB | 1677 KB | +1500 KB |
| `prometheus,otlp` | Prometheus + OpenTelemetry (lib only) | 177 KB | 177 KB | 0 KB (baseline) |
| `prometheus,otlp,standalone` | Prometheus + OpenTelemetry + standalone | 1.65 MB | 1694 KB | +1517 KB |
| `full` | Full (all features) (lib only) | 327 KB | 327 KB | +150 KB |

### Minimal Build

For the smallest binary size:

```toml
[dependencies]
observability-kit = { version = "0.1", default-features = false, features = ["prometheus"] }
```

### Full Build

For all features:

```toml
[dependencies]
observability-kit = { version = "0.1", features = ["full"] }
```

## Running the Example

```bash
# Run the standalone server example
cargo run --example standalone-prometheus --features "prometheus standalone"

# In another terminal:
curl http://127.0.0.1:9090/metrics
curl http://127.0.0.1:9090/health
curl http://127.0.0.1:9090/ready
```

## Running Tests

```bash
# Run all tests
cargo test --features mock

# Run with all features
cargo test --features full
```

## Roadmap

- [ ] OpenTelemetry/OTLP backend
- [ ] Axum middleware integration  
- [ ] Actix middleware integration
- [ ] JSON/YAML configuration
- [ ] Fake data generator for testing

## License

MIT License - see [LICENSE](LICENSE) for details.
