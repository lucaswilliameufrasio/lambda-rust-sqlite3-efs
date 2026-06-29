resource "aws_lambda_function" "api" {
  filename         = local.zip_path
  source_code_hash = filebase64sha256(local.zip_path)
  function_name    = "${local.prefix}-api"
  role             = aws_iam_role.api.arn
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  timeout          = 7
  memory_size      = 128
  architectures    = ["arm64"]

  layers = ["arn:aws:lambda:${local.region}:753240598075:layer:LambdaAdapterLayerArm64:28"]

  environment {
    variables = {
      TZ        = "UTC"
      RUST_LOG  = "info"

      DATABASE_URL = "sqlite:/mnt/volume/users.db"
      DATABASE_PATH = "/mnt/volume/users.db"

      AWS_LAMBDA_EXEC_WRAPPER      = "/opt/bootstrap"
      RUST_LOG                     = "info"
      PORT                         = 9980
      AWS_LWA_READINESS_CHECK_PORT = 9980
      AWS_LWA_READINESS_CHECK_PATH = "/health-check"
      AWS_LWA_ASYNC_INIT           = true
      AWS_LWA_INVOKE_MODE          = "response_stream"
      SQS_QUEUE_URL                = aws_sqs_queue.writer_queue.url
    }
  }

  file_system_config {
    arn              = aws_efs_access_point.test.arn
    local_mount_path = "/mnt/volume"
  }

  vpc_config {
    subnet_ids         = [aws_subnet.default.id]
    security_group_ids = [aws_security_group.lambda.id]
  }

  lifecycle {
    ignore_changes = [
      source_code_hash,
    ]
  }

  tags = local.tags

  depends_on = [
    aws_iam_role.api,
    aws_efs_access_point.test,
    aws_sqs_queue.writer_queue,
  ]
}

resource "aws_lambda_function_url" "api" {
  function_name      = aws_lambda_function.api.function_name
  authorization_type = "NONE"
  invoke_mode        = "RESPONSE_STREAM"
}

resource "aws_lambda_function" "writer" {
  filename         = local.zip_path_writer
  source_code_hash = filebase64sha256(local.zip_path_writer)
  function_name    = "${local.prefix}-writer"
  role             = aws_iam_role.writer.arn
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  timeout          = 30
  memory_size      = 128
  architectures    = ["arm64"]

  layers = ["arn:aws:lambda:${local.region}:753240598075:layer:LambdaAdapterLayerArm64:28"]

  environment {
    variables = {
      TZ        = "UTC"
      RUST_LOG  = "info"

      DATABASE_URL = "sqlite:/mnt/volume/users.db"
      DATABASE_PATH = "/mnt/volume/users.db"

      AWS_LAMBDA_EXEC_WRAPPER      = "/opt/bootstrap"
      PORT                         = 9988
      AWS_LWA_READINESS_CHECK_PROTOCOL = "tcp"
      AWS_LWA_ASYNC_INIT           = true
      AWS_LWA_INVOKE_MODE          = "buffered"
    }
  }

  file_system_config {
    arn              = aws_efs_access_point.test.arn
    local_mount_path = "/mnt/volume"
  }

  vpc_config {
    subnet_ids         = [aws_subnet.default.id]
    security_group_ids = [aws_security_group.lambda.id]
  }

  lifecycle {
    ignore_changes = [
      source_code_hash,
    ]
  }

  tags = local.tags

  depends_on = [
    aws_iam_role.writer,
    aws_efs_access_point.test,
  ]
}

resource "aws_lambda_event_source_mapping" "writer_sqs" {
  event_source_arn = aws_sqs_queue.writer_queue.arn
  function_name    = aws_lambda_function.writer.arn
  batch_size       = 1
  enabled          = true
}
