#!/bin/bash

RABBITMQ_URL="amqp://observability:local_dev_only@localhost:5672"
QUEUE_NAME="telemetry"
EXCHANGE=""
ROUTING_KEY="telemetry"

echo "Publishing test messages to RabbitMQ..."

for i in {1..5}; do
    MESSAGE="{\"type\":\"log\",\"service\":\"test-app\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"message\":\"Test message $i\"}"
    
    curl -s -u observability:local_dev_only \
        -X POST \
        -H "Content-Type: application/json" \
        -d "{\"properties\":{},\"routing_key\":\"$ROUTING_KEY\",\"payload\":\"$MESSAGE\",\"payload_encoding\":\"string\"}" \
        http://localhost:15672/api/exchanges/%2F/amq.default/publish
    
    echo "Published message $i"
    sleep 1
done

echo "Done!"
