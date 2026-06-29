resource "aws_sqs_queue" "writer_queue" {
  name                       = "${local.prefix}-writer-queue"
  delay_seconds              = 0
  max_message_size           = 262144
  message_retention_seconds  = 86400
  receive_wait_time_seconds  = 0
  visibility_timeout_seconds = 60

  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.writer_dlq.arn
    maxReceiveCount     = 5
  })

  tags = local.tags
}

resource "aws_sqs_queue" "writer_dlq" {
  name = "${local.prefix}-writer-dlq"
  tags = local.tags
}
