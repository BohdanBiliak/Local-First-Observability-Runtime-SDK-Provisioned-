use lapin::{options::*, BasicProperties, Connection, ConnectionProperties};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "amqp://observability:local_dev_only@localhost:5672";
    
    println!("Connecting to RabbitMQ...");
    let connection = Connection::connect(url, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;
    
    println!("Publishing test messages...");
    
    for i in 1..=5 {
        let message = format!(
            r#"{{"type":"log","service":"test-app","message":"Test message {}"}}"#,
            i
        );
        
        channel
            .basic_publish(
                "",
                "telemetry",
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default(),
            )
            .await?;
        
        println!("Published message {}", i);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    println!("Done!");
    connection.close(200, "Normal shutdown").await?;
    Ok(())
}
