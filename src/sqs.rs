use aws_sdk_sqs::Client as SqsClient;

pub async fn publish_message(
    queue_url: &str,
    message_body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = aws_config::load_from_env().await;
    let client = SqsClient::new(&config);

    client
        .send_message()
        .queue_url(queue_url)
        .message_body(message_body)
        .send()
        .await?;

    Ok(())
}
