#!/usr/bin/env bash
set -euo pipefail

echo "=== Floci Smoke Test ==="

# Check for floci
FLOCI_CMD=""
if command -v floci &>/dev/null; then
  FLOCI_CMD="floci"
elif command -v docker &>/dev/null; then
  echo "Using docker for floci"
else
  echo "Neither floci CLI nor docker found. Skipping smoke test."
  echo "Install floci from https://github.com/floci-io/floci"
  exit 0
fi

cleanup() {
  echo "=== Cleaning up ==="
  if [ -n "${FLOCI_CONTAINER:-}" ]; then
    docker stop "$FLOCI_CONTAINER" 2>/dev/null || true
    docker rm "$FLOCI_CONTAINER" 2>/dev/null || true
  fi
}
trap cleanup EXIT

echo "=== Starting floci ==="
FLOCI_CONTAINER=$(docker run -d --rm -p 4566:4566 floci/floci:latest)
sleep 3

export AWS_ENDPOINT_URL=http://localhost:4566
export AWS_DEFAULT_REGION=us-east-1
export AWS_ACCESS_KEY_ID=test
export AWS_SECRET_ACCESS_KEY=test
export AWS_PAGER=""

echo "=== Creating SQS queue ==="
QUEUE_URL=$(aws --endpoint-url http://localhost:4566 sqs create-queue --queue-name test-queue --query 'QueueUrl' --output text)
echo "Queue URL: $QUEUE_URL"

echo "=== Sending message ==="
aws --endpoint-url http://localhost:4566 sqs send-message \
  --queue-url "$QUEUE_URL" \
  --message-body '{"id":"test123","name":"smoke-test","email":"smoke@example.com"}' \
  --output json

echo "=== Receiving message ==="
MESSAGE=$(aws --endpoint-url http://localhost:4566 sqs receive-message \
  --queue-url "$QUEUE_URL" \
  --wait-time-seconds 2 \
  --output json)

if echo "$MESSAGE" | jq -e '.Messages[0].Body' >/dev/null 2>&1; then
  BODY=$(echo "$MESSAGE" | jq -r '.Messages[0].Body')
  echo "Received: $BODY"
  if [ "$BODY" = '{"id":"test123","name":"smoke-test","email":"smoke@example.com"}' ]; then
    echo "=== Smoke test PASSED ==="
  else
    echo "=== Smoke test FAILED: unexpected body ==="
    exit 1
  fi
else
  echo "=== Smoke test FAILED: no message received ==="
  exit 1
fi
