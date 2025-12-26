## Project Structure

```
src/
├── main.rs              # Entry point, runtime setup
├── lib.rs               # Library exports
├── config/              # Configuration management
├── messaging/           # RabbitMQ consumer
│   ├── consumer.rs      # AMQP connection and consumption
│   └── handler.rs       # Message routing
├── processors/          # Event processors
│   ├── traits.rs        # EventProcessor trait
│   └── log_processor.rs # Log event handling
├── adapters/            # External service clients
│   └── loki.rs          # Loki HTTP client
└── contracts/           # Event type definitions
```

## Development

```bash
# Build
cargo build

# Run
cargo run

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```
