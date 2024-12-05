provider "aws" {
  region = var.aws_region
}

variable "aws_region" {
  description = "AWS region for all resources."
  type        = string
  default     = "us-east-1"
}

data "aws_caller_identity" "current" {}

data "aws_region" "current" {}

locals {
  prefix = "lambda-rust-sqlite3-efs"

  account_id = data.aws_caller_identity.current.account_id

  region = data.aws_region.current.name

  zip_path               = "../bootstrap.zip"

  og_resources_of_this_project_arn = "arn:aws:logs:${local.region}:${local.account_id}:log-group:/aws/lambda/${local.prefix}*:*"
  log_groups_of_this_project_arn    = "arn:aws:logs:${local.region}:${local.account_id}:log-group:/aws/lambda/${local.prefix}*:*"

  tags = {
    Project = "Lambda Rust SQLite3 EFS"
  }
}

output "lambda_function_url" {
  value = aws_lambda_function_url.api.function_url
}