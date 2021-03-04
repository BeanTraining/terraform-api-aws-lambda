terraform {
  required_providers {
    aws = {
      source = "hashicorp/aws"
    }
    tfe = {
      version = "~> 0.24.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}
