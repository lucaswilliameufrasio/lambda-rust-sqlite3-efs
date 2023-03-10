resource "aws_efs_file_system" "foo" {
  creation_token = "any-creation-token-name-just-wanna-test"

  tags = merge(
    local.tags,
    {
      Name = "${local.prefix} efs"
    }
  )
}
