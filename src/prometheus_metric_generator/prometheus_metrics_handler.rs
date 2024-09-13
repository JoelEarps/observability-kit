use prometheus_client::{metrics::gauge::Gauge, registry::Registry};

// Lets talk through private values

#[derive(Debug)]
pub struct RegistryState {
    pub registry: Registry,
}

// state
//     .registry
//     .register("Active Connections", "Active number of connections between all clients and all servers", metrics.active_connections.clone());
// Pass through all metrics required with the following:
/* 
1. Name:
2. Description
3. Initial Value
*/

// API functions to change - find via name - structure name? Imply these functions for primitive type? Atrributes might be better?
// Increment by one
// Increment by a number
// Decrement by one
// Decrement by a certain number

#[derive(Debug)]
pub struct Metrics {
    pub active_connections: Gauge<i64>,
}

impl Metrics {
    pub fn inc_active_connections(&self) {
        self.active_connections.inc_by(4);
    }
}

pub(crate) struct PrometheusMetricHandler {
    pub all_metrics: Metrics,
    pub registry_state: RegistryState
}

impl PrometheusMetricHandler {
    pub fn new()-> Self {
        // pass in metrics desired for metrics and the generator can add them to the registry
        PrometheusMetricHandler {
            all_metrics: Metrics { active_connections: Default::default() },
            registry_state: RegistryState { registry: Default::default() }
        }
    }

    fn create_mutexes(){

    }

    // Access and lock for thread safety?
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_metric_handler_is_valid(){
        let test_metrics = Metrics{
            active_connections: Default::default(),
        };
        let test_default = test_metrics.active_connections.get();
        assert_eq!(test_default, 0);
    }
}