# Tokio

## Spawning Tasks

This is done using a JoinSet - a joinset is a collection of tasks
The way tasks are spawned and managed are by the following:

1. JoinSet::spawn - spawns a task and passes it to the set to be executed
2. JoinSet::join_next - waits for the a task to complete and then manage the result

## Custom Error Handling with the JoinSet

What should be the

## Passing state throughout the application

# Prometheus

## Promtheus Client Library

Registry
Application State

# Kubernetes

## Readiness

## Healthiness

## Metrics

### Metric Types

Counters
Gauges

## Rust notes

release Please

Prometheus

Counters
Histograms
Labels and families

Open telemetry

Release please


rlib file

Measuring binary size




From a compiler point of view

How does this work:

// For JSON files, pre-parse size if possible, or use reasonable capacity
    #[cfg(feature = "json-config")]
    pub fn from_json_file_fast(path: impl AsRef<std::path::Path>) -> Result<Self, DeserializeError> {
        use std::fs::File;
        use std::io::BufReader;

// What is deserialisation

// How does serde implement deserialisation, how much work would it be to do this myself


Examples of registry stuff


Simple for loop, is there a better way to implement deserialise from the vector for speed?


## Working with std::fs and files

Not allowing symlinks

Only allowing certain links

is_file()

extension()

cannonicalize()

What is a path bugf

### Working with chars

isalphanumeric()

Chars<'_>
