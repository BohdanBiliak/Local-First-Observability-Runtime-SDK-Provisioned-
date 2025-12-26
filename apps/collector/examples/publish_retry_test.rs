use lapin::{options::*, BasicProperties, Connection, ConnectionProperties};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "amqp://observability:local_dev_only@localhost:5672";
    
    println!("Connecting to RabbitMQ...");
    let connection = Connection::connect(url, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;
    
    println!("Publishing test messages with different failure scenarios...");
    
    let messages = vec![
        (r#"{"type":"log","message":"Success message 1"}"#, "Success 1"),
        (r#"{"type":"log","message":"Success message 2"}"#, "Success 2"),
        (r#"{"type":"log","fail":"transient","message":"Will retry"}"#, "Transient failure (will retry)"),
        (r#"{"type":"log","message":"Success message 3"}"#, "Success 3"),
        (r#"{"type":"log","fail":"permanent","message":"No retry"}"#, "Permanent failure (goes to DLQ)"),
        (r#"{"type":"log","message":"Success message 4"}"#, "Success 4"),
    ];
    
    for (message, description) in messages {
        channel
            .basic_publish(
                "",
                "telemetry",
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default(),
            )
            .await?;
        
        println!("✓ Published: {}", description);
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    
    println!("\nAll test messages published!");
    println!("- 4 messages should succeed");
    println!("- 1 message will retry up to 3 times then go to DLQ");
    println!("- 1 message will immediately go to DLQ");
    
    connection.close(200, "Normal shutdown").await?;
    Ok(())
}
