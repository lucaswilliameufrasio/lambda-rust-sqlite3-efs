
## Lambda layer used on this project to run the app without an API Gateway event adapter

https://github.com/awslabs/aws-lambda-web-adapter

## How to run migrations

```
export DATABASE_URL="sqlite:users.db"
sqlx migrate run
```

## How to revert migrations

```
export DATABASE_URL="sqlite:users.db"
sqlx migrate revert
```

## How to create migrations

```
sqlx migrate add -r add_i_dont_know_table
```

## Just use AWS DataSync to backup the database and be happy

https://repost.aws/knowledge-center/datasync-transfer-efs-s3
