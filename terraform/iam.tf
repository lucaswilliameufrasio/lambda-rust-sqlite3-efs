resource "aws_iam_role" "lambda" {
  name               = "${local.prefix}-lambda-iam-role"
  description        = "Assume that the executor has the permission for executing operations as a lambda"
  assume_role_policy = jsonencode({
    "Version": "2012-10-17",
    "Statement": [
      {
        "Effect": "Allow",
        "Principal": {
          "Service": ["lambda.amazonaws.com"]
        },
        "Action": "sts:AssumeRole"
      }
    ]
  })


  tags = merge(
    local.tags,
    {
      Name = "${local.prefix}-lambda-iam-role"
    }
  )
}

data "aws_iam_policy_document" "lambda" {
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
      "ec2:UnassignPrivateIpAddresses"
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
    sid = "AccessEFS"
    effect = "Allow"
    actions = [
      "elasticfilesystem:ClientMount",
          "elasticfilesystem:ClientWrite"
    ]
    resources = [
      aws_efs_access_point.test.arn
    ]
  }

}

resource "aws_iam_policy" "lambda" {
  name = "${local.prefix}-lambda-iam-policy"
  policy = data.aws_iam_policy_document.lambda.json
}


resource "aws_iam_role_policy_attachment" "lambda" {
  policy_arn = aws_iam_policy.lambda.arn
  role       = aws_iam_role.lambda.name
}
