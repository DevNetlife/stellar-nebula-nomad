# Remote state backend (issue #515). Backend config can't take variables, so
# the bucket/table/region below are placeholders — set them for your account
# via `terraform init -backend-config=...` per environment, or edit in place.
terraform {
  backend "s3" {
    bucket         = "nebula-nomad-terraform-state"
    key            = "global/terraform.tfstate"
    region         = "us-east-1"
    dynamodb_table = "nebula-nomad-terraform-locks"
    encrypt        = true
  }
}
