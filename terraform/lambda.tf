resource "aws_lambda_function" "api" {
  filename         = local.zip_path
  source_code_hash = filebase64sha256(local.zip_path)
  function_name    = "${local.prefix}-api"
  role             = aws_iam_role.lambda.arn
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  timeout          = 7
  memory_size      = 128
  architectures    = ["arm64"]

  layers = ["arn:aws:lambda:${local.region}:753240598075:layer:LambdaAdapterLayerArm64:23"]

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
    }

    
  }

file_system_config {
    arn            = aws_efs_access_point.test.arn
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
    aws_iam_role.lambda,
    aws_efs_access_point.test
  ]
}

resource "aws_lambda_function_url" "api" {
  function_name      = aws_lambda_function.api.function_name
  authorization_type = "NONE"
  invoke_mode        = "RESPONSE_STREAM"
}
