use std::default;

use prometheus_client::metrics::{counter::{Counter, Atomic}, gauge::Gauge};

/*
Input = vector of metrics structs

Aim: Generate vector types, perform prometheus logic required and set up access control
*/

// This would mean that all Metrics would technically need to check if they could perform that operation - as they could be one of several types
// And the functions may cause undefined behaviour

// Into or from

enum SupportedMetrics {
    counter,
    gauge
}

#[derive(Debug)]
enum MetricType {
    WrappedCounter(Counter),
    WrappedGauge(Gauge)
}

impl Default for MetricType {
    fn default() -> Self {
        MetricType::WrappedCounter(
            Default::default()
        )
    }
}


impl MetricType {
    fn new_metric(input: SupportedMetrics) -> MetricType{
        // DO not need custom Metric Here as you will only be able to pass the enum
        match input {
            SupportedMetrics::counter => {
                println!("Creating Counter");
                MetricType::WrappedCounter(Default::default())
            },
            SupportedMetrics::gauge => {
                println!("Creating Gauge");
                MetricType::WrappedGauge(Default::default())
            },
        }
    }

    // What have we done here
    fn inc(&self){
         if let MetricType::WrappedCounter(counter) = self {
            counter.inc();  // Assuming `counter` implements the `inc` method
    }}

    fn get_value(&self) -> u64{
        if let MetricType::WrappedCounter(counter) = self {
            println!("Testing testing 123");
            let test_stuff = counter.get();  // Assuming `counter` implements the `get` method
            println!("{}", test_stuff);
            return test_stuff
    } else {
        100
    }
    }
}


struct Metric {
    metric_type: MetricType,
    title: String,
    description: String,
    value: i32
}


trait BasicMetricOperations {
    fn reset_to_zero(&self) -> ();
    fn increment_by_one(&self) -> ();
    fn increment_by_custom_value(&self, increment: i32) -> ();
}


impl BasicMetricOperations for Metric {
    fn reset_to_zero(&self) -> () {
        println!("Resetting Metric of {:?} to 0", self.title);
        
    }
    // When first implementing we could not implement the increment function as this does not allow us to find the increment function as the Metric type doesn't
    // Imply the right functions
    fn increment_by_one(&self) -> (){
        println!("Incrementing {:?} by 1", self.title);
        self.metric_type.inc();
    }
    fn increment_by_custom_value(&self, increment: i32) -> (){
        println!("Incrementing {:?} by {}", self.title, increment);
    }
}


trait GaugeFunctions {
    fn decrement_by_one(&self) -> ();
}

impl GaugeFunctions for Metric {
    fn decrement_by_one(&self) -> () {
        // Check this is a not a counter and only allow for non counters
        // Custom Error?
        println!("Decrementing {:?} by 1", self.title);
    }
}


// Test Scenarios
/*
1. Create Counter
    a. Increment by one and assert on value
    b. Increment by custom val and assert by value
2. Create Gauge
    a. Increment by one and assert on value
    b. Increment by custom val and assert by value
    c. Decrement by one and assert on value
    d. Decrement by custom val and assert by value
3. Create array of types based on dict input and then implement a find function to perform the actions 
 */
#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_metric_type_counter(){
        let test_metric = Metric{
            metric_type: MetricType::new_metric(SupportedMetrics::counter),
            title: "Test Counter".to_string(),
            description: ("This is a test Counter").to_string(),
            value : 0
        };
        println!("Test Value: {} and start value {:?}", test_metric.title, test_metric.metric_type.get_value());
        test_metric.increment_by_one();
        println!("Test Value: {} and start value {:?}", test_metric.title, test_metric.metric_type.get_value());
    }

    #[test]
     fn test_metric_type_gauge(){
        let test_metric_gauge = Metric{
            metric_type: MetricType::new_metric(SupportedMetrics::gauge),
            title: "Test Gauge".to_string(),
            description: ("This is a test Counter").to_string(),
            value : 0
        };
        println!("Test Value: {:?}", test_metric_gauge.metric_type);
    }
}
