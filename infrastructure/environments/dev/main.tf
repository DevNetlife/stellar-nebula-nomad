# dev environment — issue #515. Wires the shared modules; run with
# `terraform init -backend-config=../../backend-dev.hcl` against this
# directory, or via a workspace (`terraform workspace select dev`).

terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

variable "aws_region" {
  type    = string
  default = "us-east-1"
}

variable "vpc_id" {
  type = string
}

variable "subnet_ids" {
  type = list(string)
}

variable "db_password" {
  type      = string
  sensitive = true
}

module "monitoring" {
  source      = "../../modules/monitoring"
  environment = "dev"
  vpc_id      = var.vpc_id
  subnet_id   = var.subnet_ids[0]
}

module "database" {
  source      = "../../modules/database"
  environment = "dev"
  vpc_id      = var.vpc_id
  subnet_ids  = var.subnet_ids
  db_password = var.db_password
}

module "secrets" {
  source      = "../../modules/secrets"
  environment = "dev"
}
