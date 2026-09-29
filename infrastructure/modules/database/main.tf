# Database module: RDS Postgres (analytics) + ElastiCache Redis (caching) —
# issue #515.

variable "environment" {
  type = string
}

variable "vpc_id" {
  type = string
}

variable "subnet_ids" {
  type = list(string)
}

variable "postgres_instance_class" {
  type    = string
  default = "db.t3.micro"
}

variable "redis_node_type" {
  type    = string
  default = "cache.t3.micro"
}

variable "db_password" {
  type      = string
  sensitive = true
}

resource "aws_db_subnet_group" "analytics" {
  name       = "nebula-nomad-${var.environment}-db"
  subnet_ids = var.subnet_ids
}

resource "aws_security_group" "db" {
  name_prefix = "nebula-nomad-db-${var.environment}-"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/8"]
  }

  ingress {
    from_port   = 6379
    to_port     = 6379
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/8"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_db_instance" "analytics" {
  identifier             = "nebula-nomad-analytics-${var.environment}"
  engine                 = "postgres"
  engine_version         = "16"
  instance_class         = var.postgres_instance_class
  allocated_storage      = 20
  db_name                = "nebula_analytics"
  username               = "nebula_admin"
  password               = var.db_password
  db_subnet_group_name   = aws_db_subnet_group.analytics.name
  vpc_security_group_ids = [aws_security_group.db.id]

  backup_retention_period = 7
  skip_final_snapshot     = var.environment != "prod"
  storage_encrypted       = true

  tags = {
    Environment = var.environment
  }
}

resource "aws_elasticache_subnet_group" "cache" {
  name       = "nebula-nomad-${var.environment}-cache"
  subnet_ids = var.subnet_ids
}

resource "aws_elasticache_cluster" "cache" {
  cluster_id           = "nebula-nomad-${var.environment}"
  engine               = "redis"
  node_type            = var.redis_node_type
  num_cache_nodes      = 1
  parameter_group_name = "default.redis7"
  subnet_group_name    = aws_elasticache_subnet_group.cache.name
  security_group_ids   = [aws_security_group.db.id]
}

output "postgres_endpoint" {
  value = aws_db_instance.analytics.endpoint
}

output "redis_endpoint" {
  value = aws_elasticache_cluster.cache.cache_nodes[0].address
}
