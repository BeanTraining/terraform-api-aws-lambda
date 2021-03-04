terraform {
  required_providers {
    aws = {
      source = "hashicorp/aws"
    }
    tfe = {
      version = "~> 0.24.0"
    }
  }

  required_version = "~> 0.14"

  backend "remote" {
    organization = "BeanTraining"

    workspaces {
      name = "terraform-api-aws-lambda"
    }
  }

}

provider "aws" {
  region = var.aws_region
}
