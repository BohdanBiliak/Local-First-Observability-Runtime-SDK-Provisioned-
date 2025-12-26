#!/usr/bin/env python3
import pika
import json
import sys

credentials = pika.PlainCredentials('observability', 'local_dev_only')
parameters = pika.ConnectionParameters('localhost', 5672, '/', credentials)

connection = pika.BlockingConnection(parameters)
channel = connection.channel()

channel.queue_declare(queue='telemetry', durable=True)

for i in range(1, 6):
    message = {
        "type": "log",
        "service": "test-app",
        "message": f"Test message {i}"
    }
    
    channel.basic_publish(
        exchange='',
        routing_key='telemetry',
        body=json.dumps(message)
    )
    print(f"Published message {i}")

connection.close()
print("Done!")
