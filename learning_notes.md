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
