
resource "aws_subnet" "subnet_for_lambda" {
  vpc_id     = aws_vpc.vpc_for_lambda.id
  cidr_block = "10.0.1.0/24"
}
resource "aws_vpc" "vpc_for_lambda" {
  cidr_block = "10.0.0.0/16"
}
resource "aws_security_group" "sg_for_lambda" {
  name        = "sg_for_lambdaV1"
  description = "sg_for_lambda"
  vpc_id      = aws_vpc.vpc_for_lambda.id
  ingress {
    description = "NFS"
    from_port   = 2049
    to_port     = 2049
    protocol    = "tcp"
    cidr_blocks = [aws_vpc.vpc_for_lambda.cidr_block]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
  lifecycle {
    create_before_destroy = true
  }
}
