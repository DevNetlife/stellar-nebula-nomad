# Infrastructure Runbook

Infrastructure is defined as code under `infrastructure/` (issue #515).

## Layout

- `infrastructure/modules/` — reusable modules: `monitoring` (Prometheus/Grafana host), `database` (RDS Postgres + ElastiCache Redis), `secrets` (AWS Secrets Manager). Networking (`main.tf`), the API Lambda (`lambda.tf`, `api_gateway.tf`), WAF (`waf.tf`), and contract-state backups (`backup/`) predate this issue and live at the `infrastructure/` root.
- `infrastructure/environments/{dev,staging,prod}/` — one root module per environment, wiring the shared modules together with environment-specific variables.
- `infrastructure/backend.tf` — remote state (S3 + DynamoDB lock). Update the bucket/table names for your AWS account before first `init`.

## Common operations

**Plan a change for an environment:**
```sh
cd infrastructure/environments/<env>
terraform init
terraform plan -var-file=terraform.tfvars
```

**Apply:**
```sh
terraform apply -var-file=terraform.tfvars
```

Always run `plan` and read the diff before `apply` — especially in `prod`.

**Format/validate before committing:**
```sh
terraform fmt -recursive infrastructure/
terraform validate
```

## Notes / scope

This is infrastructure-as-*code*: the modules define what would be deployed, but nothing here has been applied against a real AWS account from this change — there's no account to apply it to from a coding session. Before first real use: fill in each environment's `terraform.tfvars` (VPC/subnet IDs, DB password sourced from the `secrets` module, not committed), point `backend.tf` at a real state bucket/lock table, and run `terraform plan` to confirm the diff before anyone applies it.
