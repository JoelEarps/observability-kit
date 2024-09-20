# Version 1 Acceptance Criteria

1. Works for prometheus metric scraping only
2. Simple implementation for Counter and Gauge Metrics
3. Simple Readiness and Health Endpoints
4. Implement more complex metric types e.g. Histogram, Summary and Family (custom metrics)
5. Way to gracefully shut down - return thread or just await to start with

## Other Considerations

1. RWLock over Mutex
2. Event Streaming
3. Allow
