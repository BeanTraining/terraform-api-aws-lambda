
resource "aws_subnet" "subnet_for_lambda" {
  vpc_id     = aws_vpc.vpc_for_lambda.id
  cidr_block = "10.0.1.0/24"
}

resource "aws_vpc" "vpc_for_lambda" {
  cidr_block = "10.0.0.0/16"
  assign_generated_ipv6_cidr_block = true
    tags = {
    Name = "tfe_api_vpc_for_lambda"
   }
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

resource "aws_route_table" "rt_for_lambda" {
  vpc_id = aws_vpc.vpc_for_lambda.id

  route {
    ipv6_cidr_block        = "::/0"
    egress_only_gateway_id = aws_egress_only_internet_gateway.eoig_for_lambda.id
  }

  tags = {
    Name = "tfe_api_vpc_for_lambda"
  }
}

resource "aws_egress_only_internet_gateway" "eoig_for_lambda" {
  vpc_id = aws_vpc.vpc_for_lambda.id

  tags = {
    Name = "tfe_api_vpc_for_lambda"
  }
}

resource "aws_route_table_association" "rta_for_lambda" {
  subnet_id      = aws_subnet.subnet_for_lambda.id
  route_table_id = aws_route_table.rt_for_lambda.id
}

resource "aws_main_route_table_association" "mrta_for_lambda" {
  vpc_id         = aws_vpc.vpc_for_lambda.id
  route_table_id = aws_route_table.rt_for_lambda.id
}

