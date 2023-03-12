resource "aws_ssm_parameter" "efs_arn" {
  name      = "/${local.prefix}/default/EFS_ARN"
  type      = "String"
  value     = aws_efs_file_system.foo.arn
  tags      = local.tags
  overwrite = true

  depends_on = [
    aws_efs_access_point.test
  ]
}

resource "aws_ssm_parameter" "efs_ap_arn" {
  name      = "/${local.prefix}/default/EFS_AP_ARN"
  type      = "String"
  value     = aws_efs_access_point.test.arn
  tags      = local.tags
  overwrite = true

  depends_on = [
    aws_efs_access_point.test
  ]
}

resource "aws_ssm_parameter" "vpc_sg_ids" {
  name      = "/${local.prefix}/default/VPC_SG_IDS"
  type      = "StringList"
  value     = aws_security_group.lambda.id
  tags      = local.tags
  overwrite = true
}

resource "aws_ssm_parameter" "vpc_subnet_ids" {
  name      = "/${local.prefix}/default/VPC_SUBNET_IDS"
  type      = "StringList"
  value     = aws_subnet.default.id
  tags      = local.tags
  overwrite = true
}

