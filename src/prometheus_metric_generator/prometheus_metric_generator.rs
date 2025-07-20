use std::{default, fmt::Display, str::FromStr};

use prometheus_client::metrics::{
    counter::{self, Counter},
    gauge::Gauge,
    MetricType,
};

// There is a trait called atomic operations, which means we cna trait bound the methods
// use the type and precision with deserialisation to figure out what to cast for
//
struct BaseMetric<A> {
    metric: A,
    title: String,
    description: String,
}

/// Associates metric types with their value types
/// As we cannot return multiple types from a function, rust does not support it
/// We have created an associated type - static the type returns for each implementer for the trait
/// Currently we support Counter and Gauges - the two most common types which require u64 and i64 respectively
pub trait MetricValueType {
    type Value;
}

/// The implementation for the relevant supported metric types
/// This tells the compiler:
// "For a Counter<u64>, its value type is u64."
// "For a Gauge<i64>, its value type is i64."
// So now, anywhere in your code, if you're working with something that implements MetricValueType, you can get its value type like this:
impl MetricValueType for Counter<u64> {
    type Value = u64;
}

impl MetricValueType for Gauge<i64> {
    type Value = i64;
}
/// The where trait is bounding the generic type A to must having implemented the MetricValueType trait
/// ` <A as MetricValueType>::Value` That’s how you reference the associated type on the trait MetricValueType for the type A.
pub trait BasicMetricOperations<A>
where
    A: MetricValueType,
{
    fn new(metric_name: &str, metric_description: &str, metric: A) -> Self;
    fn increment_by_one(&self);
    fn increment_by_custom_value(&self, increment: <A as MetricValueType>::Value);
    fn get_metric_value(&self) -> <A as MetricValueType>::Value;
}

/// Counter Functionality
impl BasicMetricOperations<Counter<u64>> for BaseMetric<Counter<u64>> {
    fn new(metric_name: &str, metric_description: &str, metric: Counter<u64>) -> Self {
        BaseMetric {
            metric,
            title: metric_name.to_string(),
            description: metric_description.to_string(),
        }
    }

    fn increment_by_one(&self) {
        self.metric.inc();
    }

    fn increment_by_custom_value(&self, increment: u64) {
        self.metric.inc_by(increment);
    }

    fn get_metric_value(&self) -> u64 {
        self.metric.get()
    }
}

// Gauge Functionality

/// What does this mean?
trait GaugeMetricFunctionality<U>: BasicMetricOperations<U>
where
    U: MetricValueType,
{
    fn reset_to_zero(&self);
    fn decrement_by_one(&self);
    fn decrement_by_custom_value(&self, decrement: <U as MetricValueType>::Value);
    fn set_to_custom_value(&self, desired_value: <U as MetricValueType>::Value);
}

impl BasicMetricOperations<Gauge<i64>> for BaseMetric<Gauge<i64>> {
    fn new(metric_name: &str, metric_description: &str, metric: Gauge<i64>) -> Self {
        BaseMetric {
            metric,
            title: metric_name.to_string(),
            description: metric_description.to_string(),
        }
    }

    fn increment_by_one(&self) {
        self.metric.inc();
    }

    fn increment_by_custom_value(&self, increment: i64) {
        self.metric.inc_by(increment);
    }

    fn get_metric_value(&self) -> i64 {
        self.metric.get()
    }
}

impl GaugeMetricFunctionality<Gauge<i64>> for BaseMetric<Gauge<i64>> {
    fn reset_to_zero(&self) {
        self.metric.set(0);
    }

    fn decrement_by_one(&self) {
        self.metric.dec();
    }

    fn decrement_by_custom_value(&self, decrement: i64) {
        self.metric.dec_by(decrement);
    }

    fn set_to_custom_value(&self, desired_value: i64) {
        self.metric.set(desired_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_type_counter() {
        let test_counter = Counter::default();
        let test_metric = BaseMetric::new(
            "test_metric_counter",
            "A metric for declaring a counter",
            test_counter,
        );
        assert_eq!(test_metric.get_metric_value(), 0);
        test_metric.increment_by_one();
        assert_eq!(test_metric.get_metric_value(), 1);
        test_metric.increment_by_custom_value(20);
        assert_eq!(test_metric.get_metric_value(), 21);
    }

    #[test]
    fn test_metric_type_gauge() {
        let test_gauge = Gauge::default();
        let test_metric_gauge = BaseMetric::new(
            "test_metric_counter",
            "A metric for declaring a counter",
            test_gauge,
        );
        assert_eq!(test_metric_gauge.get_metric_value(), 0);
        test_metric_gauge.increment_by_one();
        assert_eq!(test_metric_gauge.get_metric_value(), 1);
        test_metric_gauge.increment_by_custom_value(20);
        assert_eq!(test_metric_gauge.get_metric_value(), 21);
        test_metric_gauge.decrement_by_one();
        assert_eq!(test_metric_gauge.get_metric_value(), 20);
        test_metric_gauge.decrement_by_custom_value(10);
        assert_eq!(test_metric_gauge.get_metric_value(), 10);
        test_metric_gauge.reset_to_zero();
        assert_eq!(test_metric_gauge.get_metric_value(), 0);
        test_metric_gauge.set_to_custom_value(500);
        assert_eq!(test_metric_gauge.get_metric_value(), 500);
    }
}
