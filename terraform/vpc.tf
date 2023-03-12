data "aws_vpc" "default" {
  default = true
}

resource "aws_security_group" "lambda" {
  name        = "${local.prefix}-lambda-sg"
  vpc_id      = data.aws_vpc.default.id
  description = "Allow outbound traffic"

  egress {
    description = "all"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = local.tags
}

resource "aws_subnet" "default" {
  vpc_id                  = data.aws_vpc.default.id
  cidr_block              = "172.31.100.0/24"
  availability_zone       = "us-east-1a"
  map_public_ip_on_launch = false

  tags = merge(local.tags, {
    Name = "${local.prefix}-private-subnet-us-east-1a"
  })
}

resource "aws_security_group" "efs_sg" {
  name        = "${local.prefix}-efs-sg"
  description = "A security group for Amazon EFS that allows inbound NFS access from resources (including the mount target) associated with this security group (TCP 2049)."
  vpc_id      = data.aws_vpc.default.id

  ingress {
    from_port   = 2049
    to_port     = 2049
    protocol    = "tcp"
    cidr_blocks = ["172.31.0.0/16"]
    description = "Allow NFS traffic - TCP 2049"
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

}
