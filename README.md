# Local Observability Runtime

A lightweight monorepo for local observability infrastructure with distributed tracing, logging, and metrics.

## Stack

- **NestJS** - API gateway and orchestration
- **Rust** - High-performance log collector
- **RabbitMQ** - Message broker for event streaming
- **Loki** - Log aggregation
- **Prometheus** - Metrics collection
- **Docker** - Containerized infrastructure

## Structure

```
apps/
  api/         # NestJS API gateway
  collector/   # Rust log collector service
infra/
  docker/      # Docker Compose configurations
docs/
  architecture.md  # System architecture and design patterns
```

## Quick Start

```bash
# Setup environment
cd infra/docker
cp .env.example .env
# Edit .env with secure credentials

# Start infrastructure
docker-compose up -d

# API will be available at http://localhost:3000
# Prometheus at http://localhost:9090
# Loki at http://localhost:3100
```

## Documentation

See [docs/architecture.md](docs/architecture.md) for detailed architecture and design patterns.
