#!/bin/bash

BASEDIR=$(cd $(dirname $0) && pwd)

deploy_function() {
  echo -e "🔄 Updating $1 function"

  LAST_MODIFIED=$(aws lambda update-function-code --function-name "$1" --zip-file "fileb://$2" --publish --no-cli-pager | jq -r '.LastModified')

  if [ -z "$LAST_MODIFIED" ]; then
      echo "Error: Failed to retrieve last modified date for function $1"
      exit 1
  else
      echo "Last modified date: $LAST_MODIFIED"
  fi

  echo -e "✔️ $1 function was updated"
}

APP_PREFIX=lambda-rust-sqlite3-efs

deploy_function "$APP_PREFIX-api" "${BASEDIR}/../bootstrap.zip"

