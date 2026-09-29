# Monitoring stack module (Prometheus + Grafana + Alertmanager) — issue #515.
# Runs the existing monitoring/docker-compose.yml stack on a small dedicated
# instance rather than reinventing it as separate managed services.

variable "environment" {
  type = string
}

variable "instance_type" {
  type    = string
  default = "t3.medium"
}

variable "vpc_id" {
  type = string
}

variable "subnet_id" {
  type = string
}

resource "aws_security_group" "monitoring" {
  name_prefix = "nebula-nomad-monitoring-${var.environment}-"
  vpc_id      = var.vpc_id

  ingress {
    description = "Grafana"
    from_port   = 3000
    to_port     = 3000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "Prometheus (internal)"
    from_port   = 9090
    to_port     = 9090
    protocol    = "tcp"
    self        = true
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "nebula-nomad-monitoring-${var.environment}"
    Environment = var.environment
  }
}

resource "aws_instance" "monitoring" {
  ami                    = data.aws_ami.ubuntu.id
  instance_type          = var.instance_type
  subnet_id              = var.subnet_id
  vpc_security_group_ids = [aws_security_group.monitoring.id]

  # Bootstraps docker + docker-compose and runs the existing
  # monitoring/docker-compose.yml stack (Prometheus, Grafana, Alertmanager).
  user_data = <<-EOF
    #!/bin/bash
    set -euo pipefail
    apt-get update -y
    apt-get install -y docker.io docker-compose-plugin
    systemctl enable --now docker
  EOF

  tags = {
    Name        = "nebula-nomad-monitoring-${var.environment}"
    Environment = var.environment
  }
}

data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"] # Canonical

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*"]
  }
}

output "grafana_host" {
  value = aws_instance.monitoring.public_dns
}
