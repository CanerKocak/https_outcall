#!/bin/bash

# Test script for Claude API endpoint

# Load environment variables from .env file
if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
else
  echo "Error: .env file not found. Please create one with your CLAUDE_API_KEY."
  exit 1
fi

# Check if CLAUDE_API_KEY is set
if [ -z "$CLAUDE_API_KEY" ]; then
  echo "Error: CLAUDE_API_KEY is not set in .env file."
  exit 1
fi

# Set the server URL
SERVER_URL="http://localhost:8080"

# Set the canister ID and request ID
CANISTER_ID="rrkah-fqaaa-aaaaa-aaaaq-cai"
REQUEST_ID="test-request-$(date +%s)"

# Create a test request
echo "Sending test request to Claude API endpoint..."
curl -X POST "${SERVER_URL}/claude" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-api-key" \
  -d '{
    "canister_id": "'"${CANISTER_ID}"'",
    "request_id": "'"${REQUEST_ID}"'",
    "system": "You are Claude, a helpful AI assistant.",
    "messages": [
      {
        "role": "user",
        "content": "Hello, Claude! What can you tell me about the Internet Computer Protocol?"
      }
    ],
    "max_tokens": 500,
    "temperature": 0.7
  }' | jq .

# Test deduplication by sending the same request again
echo -e "\nSending the same request again to test deduplication..."
curl -X POST "${SERVER_URL}/claude" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-api-key" \
  -d '{
    "canister_id": "'"${CANISTER_ID}"'",
    "request_id": "'"${REQUEST_ID}"'",
    "system": "You are Claude, a helpful AI assistant.",
    "messages": [
      {
        "role": "user",
        "content": "Hello, Claude! What can you tell me about the Internet Computer Protocol?"
      }
    ],
    "max_tokens": 500,
    "temperature": 0.7
  }' | jq .

# Send a new request with a different request ID
NEW_REQUEST_ID="test-request-$(date +%s)"
echo -e "\nSending a new request with a different request ID..."
curl -X POST "${SERVER_URL}/claude" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-api-key" \
  -d '{
    "canister_id": "'"${CANISTER_ID}"'",
    "request_id": "'"${NEW_REQUEST_ID}"'",
    "system": "You are Claude, a helpful AI assistant.",
    "messages": [
      {
        "role": "user",
        "content": "What are the advantages of using the Internet Computer Protocol?"
      }
    ],
    "max_tokens": 500,
    "temperature": 0.7
  }' | jq . 