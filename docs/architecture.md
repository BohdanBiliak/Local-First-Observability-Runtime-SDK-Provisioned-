# Architecture

## Overview

Event-driven microservices architecture for local observability with clear separation of concerns.

## Components

### API Gateway (NestJS)
**Responsibility**: HTTP interface and service orchestration

**Patterns**:
- **Dependency Injection**: NestJS modules for loose coupling
- **Facade**: Simplifies complex observability operations
- **Graceful Shutdown**: Handles SIGTERM/SIGINT for safe container stops

### Collector (Rust)
**Responsibility**: High-throughput log ingestion and parsing

**Patterns**:
- **Adapter**: Transforms various log formats to unified schema
- **Single Responsibility**: Focused on collection, not storage
- **Strategy**: Pluggable parsers for different log formats

### Message Broker (RabbitMQ)
**Responsibility**: Async event streaming between services

**Patterns**:
- **Observer**: Pub/sub for decoupled event handling
- **Producer-Consumer**: Buffering for traffic spikes

### Storage Layer
**Loki**: Time-series log storage  
**Prometheus**: Metrics and alerting

## Data Flow

```
Logs → Collector → RabbitMQ → API → Loki/Prometheus
                       ↓
                  Event handlers
```

## Design Principles

1. **SRP**: Each service has one primary responsibility
2. **Modularity**: Services communicate via contracts (RabbitMQ exchanges)
3. **Resilience**: Message queues buffer failures
4. **Performance**: Rust for I/O-bound collection, NestJS for orchestration

## Deployment

Docker Compose for local dev. Production-ready with K8s manifests (future).
