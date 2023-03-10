resource "aws_ssm_parameter" "efs_arn" {
  name  = "/${local.prefix}/default/EFS_ARN"
  type  = "String"
  value = aws_efs_file_system.foo.arn
  tags  = local.tags
}
