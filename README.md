# Lambda Rust SQLite3

It is not recommended to access SQLite3 through a Network File System but i had a use case for it.

https://www.sqlite.org/faq.html#q5

## Requirements

- [Cargo Lambda](https://www.cargo-lambda.info/guide/getting-started.html)
- [Terraform](https://developer.hashicorp.com/terraform/install?product_intent=terraform)


## Lambda layer used on this project to run the app without an API Gateway event adapter

https://github.com/awslabs/aws-lambda-web-adapter

## How to run migrations

``` bash
export DATABASE_URL="sqlite:users.db"
sqlx migrate run
```

## How to revert migrations

``` bash
export DATABASE_URL="sqlite:users.db"
sqlx migrate revert
```

## How to create migrations

``` bash
sqlx migrate add -r add_i_dont_know_table
```

## Just run the commands below to deploy the app after provisioning all infrastructure required using Terraform
``` bash
make prepare-deploy
./scripts/deploy-functions.sh
```

## Commands to provision the infrastructure using Terraform

```bash
terraform plan -out=./tfplan
terraform apply ./tfplan
```

## Just use AWS DataSync to backup the database and be happy

https://repost.aws/knowledge-center/datasync-transfer-efs-s3
