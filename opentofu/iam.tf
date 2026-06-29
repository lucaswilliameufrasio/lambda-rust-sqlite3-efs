data "aws_iam_policy_document" "lambda_assume_role" {
  statement {
    effect = "Allow"
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
    actions = ["sts:AssumeRole"]
  }
}

resource "aws_iam_role" "api" {
  name               = "${local.prefix}-api-iam-role"
  description        = "IAM role for the API Lambda function"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume_role.json

  tags = merge(local.tags, {
    Name = "${local.prefix}-api-iam-role"
  })
}

resource "aws_iam_role" "writer" {
  name               = "${local.prefix}-writer-iam-role"
  description        = "IAM role for the writer Lambda function"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume_role.json

  tags = merge(local.tags, {
    Name = "${local.prefix}-writer-iam-role"
  })
}

data "aws_iam_policy_document" "api" {
  statement {
    sid       = "AllowEC2"
    effect    = "Allow"
    resources = ["*"]
    actions = [
      "ec2:CreateNetworkInterface",
      "ec2:DeleteNetworkInterface",
      "ec2:DescribeInstances",
      "ec2:AttachNetworkInterface",
      "ec2:DescribeNetworkInterfaces",
      "ec2:AssignPrivateIpAddresses",
      "ec2:UnassignPrivateIpAddresses",
    ]
  }

  statement {
    sid       = "AllowCreatingLogGroups"
    effect    = "Allow"
    resources = [local.log_groups_of_this_project_arn]
    actions   = ["logs:CreateLogGroup"]
  }

  statement {
    sid       = "AllowWritingLogs"
    effect    = "Allow"
    resources = [local.log_groups_of_this_project_arn]
    actions = [
      "logs:CreateLogStream",
      "logs:PutLogEvents",
    ]
  }

  statement {
    sid    = "AccessEFS"
    effect = "Allow"
    actions = [
      "elasticfilesystem:ClientMount",
      "elasticfilesystem:ClientWrite",
    ]
    resources = [aws_efs_access_point.test.arn]
  }

  statement {
    sid       = "AllowSQSSend"
    effect    = "Allow"
    resources = [aws_sqs_queue.writer_queue.arn]
    actions   = ["sqs:SendMessage"]
  }
}

data "aws_iam_policy_document" "writer" {
  statement {
    sid       = "AllowEC2"
    effect    = "Allow"
    resources = ["*"]
    actions = [
      "ec2:CreateNetworkInterface",
      "ec2:DeleteNetworkInterface",
      "ec2:DescribeInstances",
      "ec2:AttachNetworkInterface",
      "ec2:DescribeNetworkInterfaces",
      "ec2:AssignPrivateIpAddresses",
      "ec2:UnassignPrivateIpAddresses",
    ]
  }

  statement {
    sid       = "AllowCreatingLogGroups"
    effect    = "Allow"
    resources = [local.log_groups_of_this_project_arn]
    actions   = ["logs:CreateLogGroup"]
  }

  statement {
    sid       = "AllowWritingLogs"
    effect    = "Allow"
    resources = [local.log_groups_of_this_project_arn]
    actions = [
      "logs:CreateLogStream",
      "logs:PutLogEvents",
    ]
  }

  statement {
    sid    = "AccessEFS"
    effect = "Allow"
    actions = [
      "elasticfilesystem:ClientMount",
      "elasticfilesystem:ClientWrite",
    ]
    resources = [aws_efs_access_point.test.arn]
  }

  statement {
    sid       = "AllowSQSConsume"
    effect    = "Allow"
    resources = [aws_sqs_queue.writer_queue.arn]
    actions = [
      "sqs:ReceiveMessage",
      "sqs:DeleteMessage",
      "sqs:GetQueueAttributes",
    ]
  }
}

resource "aws_iam_policy" "api" {
  name   = "${local.prefix}-api-iam-policy"
  policy = data.aws_iam_policy_document.api.json
}

resource "aws_iam_policy" "writer" {
  name   = "${local.prefix}-writer-iam-policy"
  policy = data.aws_iam_policy_document.writer.json
}

resource "aws_iam_role_policy_attachment" "api" {
  policy_arn = aws_iam_policy.api.arn
  role       = aws_iam_role.api.name
}

resource "aws_iam_role_policy_attachment" "writer" {
  policy_arn = aws_iam_policy.writer.arn
  role       = aws_iam_role.writer.name
}
