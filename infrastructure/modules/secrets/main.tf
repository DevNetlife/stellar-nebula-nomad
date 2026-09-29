# Secrets management module (AWS Secrets Manager) — issue #515.

variable "environment" {
  type = string
}

variable "secret_names" {
  type    = list(string)
  default = ["db-password", "deployer-secret-key", "grafana-admin-password"]
}

resource "aws_secretsmanager_secret" "this" {
  for_each = toset(var.secret_names)

  name = "nebula-nomad/${var.environment}/${each.value}"

  tags = {
    Environment = var.environment
  }
}

output "secret_arns" {
  value = { for name, secret in aws_secretsmanager_secret.this : name => secret.arn }
}
