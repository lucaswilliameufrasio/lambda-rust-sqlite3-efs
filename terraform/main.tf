provider "aws" {
  region = var.aws_region
}

variable "aws_region" {
  description = "AWS region for all resources."
  type        = string
  default     = "us-east-1"
}

locals {
  prefix = "lambda-rust-sqlite3-efs"

  tags = {
    Project = "Lambda Rust SQLite3 EFS"
  }
}
