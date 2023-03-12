resource "aws_efs_file_system" "foo" {
  creation_token = "any-creation-token-name-just-wanna-test"

  tags = merge(
    local.tags,
    {
      Name = "${local.prefix} efs"
    }
  )
}

resource "aws_efs_access_point" "test" {
  file_system_id = aws_efs_file_system.foo.id

  posix_user {
    uid = 1000
    gid = 1000
  }

  root_directory {
    path = "/volume"

    creation_info {
      owner_uid = 1000
      owner_gid = 1000
      // change this
      permissions = 777
    }
  }

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_efs_mount_target" "alpha" {
  file_system_id = aws_efs_file_system.foo.id
  subnet_id      = aws_subnet.default.id
  security_groups = [aws_security_group.efs_sg.id]
}

