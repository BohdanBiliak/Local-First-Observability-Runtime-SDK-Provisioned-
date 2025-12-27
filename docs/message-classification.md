# Message Classification & Versioning

## Overview

This system implements **domain-driven error handling** and **message versioning** for robust event processing and replay capabilities.

## Error Classification

### Problem

Previously, error handling was implicit - the system couldn't distinguish between temporary failures (network issues) and permanent failures (invalid data).

### Solution

Explicit error classification using the `ProcessingError` enum:

```rust
pub enum ProcessingError {
    Transient { reason: String },
    Permanent { reason: String },
}
```

### Routing Logic

#### Transient Errors

- **Definition**: Temporary failures that may succeed on retry
- **Examples**: Network timeouts, service unavailable, rate limiting
- **Routing**: Retry queue → Main queue (up to MAX_RETRIES)
- **After max retries**: DLQ

#### Permanent Errors

- **Definition**: Fatal errors that won't succeed on retry
- **Examples**: Schema validation failures, unsupported versions, malformed data
- **Routing**: DLQ immediately (no retries)

### Usage in Rust

```rust
use observability_collector::messaging::{HandlerError, MessageHandler};

impl MessageHandler for MyHandler {
    async fn handle(&self, delivery: Delivery) -> Result<(), HandlerError> {
        // Transient error - will retry
        if network_timeout() {
            return Err(HandlerError::Transient("Network timeout".to_string()));
        }

        // Permanent error - goes to DLQ immediately
        if !validate_schema(&data) {
            return Err(HandlerError::Permanent("Invalid schema".to_string()));
        }

        Ok(())
    }
}
```

### Error Metadata

When messages are rejected, the following headers are added:

- `x-error-reason`: Human-readable error description
- `x-error-type`: `"transient"` or `"permanent"`
- `x-original-queue`: The queue where processing failed

This metadata is preserved in the DLQ for debugging and analysis.

---

## Message Versioning

### Problem

Without versioning:

- Schema changes break existing messages
- Replays fail after schema updates
- No way to handle multiple versions concurrently

### Solution

Event versioning via headers and deterministic routing:

```
x-event-version: v1
```

### Implementation

#### Rust Consumer

```rust
const EVENT_VERSION_HEADER: &str = "x-event-version";

impl MessageHandler for TelemetryHandler {
    async fn handle(&self, delivery: Delivery) -> Result<(), HandlerError> {
        let version = extract_version(&delivery.properties);

        match version.as_str() {
            "v1" => self.handle_v1(&payload),
            "v2" => self.handle_v2(&payload),
            _ => Err(HandlerError::Permanent(
                format!("Unsupported version: {}", version)
            ))
        }
    }
}
```

#### TypeScript Publisher

The `RabbitEventPublisher` automatically extracts the `eventVersion` from the event payload and adds it to message headers:

```typescript
const event = createLogCapturedEventV1({
  level: 'info',
  message: 'Test log',
  serviceName: 'api',
  environment: Environment.Development,
});

// Automatically includes: x-event-version: v1
await telemetryPublisher.publishTelemetryEvent(event);
```

---

## DLQ Inspection

### Enhanced DLQ Message DTO

Messages in the DLQ now include:

```typescript
interface DLQMessageDto {
  id: string;
  content: any;
  originalQueue: string;
  routingKey: string;
  timestamp?: number;
  retryCount?: number;
  headers?: Record<string, any>;
  errorReason?: string; // NEW: Why it failed
  errorType?: 'transient' | 'permanent'; // NEW: Error classification
  eventVersion?: string; // NEW: Event version (e.g., "v1")
}
```

### API Endpoints

#### List DLQ Messages

```bash
GET /dlq/telemetry?limit=10
```

Response:

```json
{
  "queue": "telemetry.dlq",
  "messageCount": 5,
  "messages": [
    {
      "id": "msg-123",
      "content": {...},
      "originalQueue": "telemetry",
      "routingKey": "telemetry.log.captured.v1",
      "errorReason": "Unsupported event version: v99",
      "errorType": "permanent",
      "eventVersion": "v99",
      "retryCount": 0
    }
  ]
}
```

#### Inspect Specific Message

```bash
GET /dlq/telemetry/msg-123
```

#### Replay Message

```bash
POST /dlq/telemetry/replay
{
  "messageId": "msg-123",
  "targetQueue": "telemetry",
  "operator": "john@example.com"
}
```

---

## Benefits

### ✅ Domain-Driven Error Handling

- Explicit error classification
- Deterministic routing decisions
- Clear intent in code

### ✅ Deterministic Routing

- Transient errors → Retry
- Permanent errors → DLQ
- No ambiguity

### ✅ Safe Replays

- Version checking prevents incompatible replays
- Old messages handled gracefully
- Migration path for schema changes

### ✅ Interview-Ready System Design

- Production-grade error handling
- Event versioning strategy
- Observable and debuggable

---

## Testing

### Test Transient Error

```bash
curl -X POST http://localhost:3000/test/telemetry \
  -H "Content-Type: application/json" \
  -d '{"fail": "transient"}'
```

### Test Permanent Error

```bash
curl -X POST http://localhost:3000/test/telemetry \
  -H "Content-Type: application/json" \
  -d '{"fail": "permanent"}'
```

### Test Unsupported Version

Manually publish a message with `x-event-version: v99` - it will be immediately rejected to DLQ with:

- `errorReason`: "Unsupported event version: v99"
- `errorType`: "permanent"

### Inspect DLQ

```bash
curl http://localhost:3000/dlq/telemetry?limit=10
```

---

## Migration Path

### Handling v1 → v2 Schema Changes

1. **Deploy v2 handler** that supports both v1 and v2
2. **Update publishers** to send v2 events
3. **Replay v1 messages** from DLQ if needed
4. **Deprecate v1** after migration period

Example handler:

```rust
match version.as_str() {
    "v1" => {
        let v1_data = parse_v1(&payload)?;
        let v2_data = migrate_v1_to_v2(v1_data);
        self.process_v2(v2_data)
    }
    "v2" => {
        let v2_data = parse_v2(&payload)?;
        self.process_v2(v2_data)
    }
    _ => Err(HandlerError::Permanent(format!("Unsupported: {}", version)))
}
```

---

## Metrics

The system tracks:

- `messages_failed_total{error_type="transient"}` - Transient failures
- `messages_failed_total{error_type="permanent"}` - Permanent failures
- `messages_retried_total` - Retry attempts
- `messages_dlq_total` - Messages sent to DLQ
- `message_processing_duration_seconds` - Processing time by outcome

View metrics:

```bash
curl http://localhost:9090/metrics
```
