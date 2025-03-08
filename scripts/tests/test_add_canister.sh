#!/bin/bash

# Test script for adding a canister to the registry and testing functionality
# This script adds a token canister to the registry and tests various API endpoints

# Color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
API_URL="http://localhost:8080"
CANISTER_ID="sk4hs-faaaa-aaaag-at3rq-cai"
OWNER_PRINCIPAL="6mot7-e3eea-fbv55-qv7ju-ap565-mwzit-ro57f-jc4q5-4thgc-hnnsf-pqe"
MODULE_HASH="5471eb4e9e70f245d8db1a1673d43ab5ff9443c6d1588f5bdf052bdc7e88f0a5"

# Function to print colored section headers
section() {
    echo -e "\n${BLUE}======== $1 ========${NC}"
}

# Function to print success messages
success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print warning messages
warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Function to print error messages and exit
error() {
    echo -e "${RED}✗ $1${NC}"
    exit 1
}

# Check if server is running
section "Checking if server is running"
if ! curl -s "$API_URL/system/status" > /dev/null; then
    error "Server is not running at $API_URL. Please start the server first."
fi
success "Server is running"

# Step 1: Add the canister to the registry
section "Adding canister to registry"
echo "Adding canister ID: $CANISTER_ID"

REGISTER_RESPONSE=$(curl -s -X POST "$API_URL/canisters" \
    -H "Content-Type: application/json" \
    -d '{
        "principal": "'"$OWNER_PRINCIPAL"'",
        "canister_id": "'"$CANISTER_ID"'",
        "canister_type": "token"
    }')

# Check if registration was successful
if echo "$REGISTER_RESPONSE" | grep -q "Canister registered successfully"; then
    success "Canister registered successfully"
else
    # If the canister already exists, it's not necessarily an error
    if echo "$REGISTER_RESPONSE" | grep -q "already exists"; then
        warning "Canister already exists in the registry"
    else
        error "Failed to register canister: $REGISTER_RESPONSE"
    fi
fi

# Step 2: Verify the canister is in the list
section "Verifying canister is in the registry"
ALL_CANISTERS=$(curl -s "$API_URL/canisters")

if echo "$ALL_CANISTERS" | grep -q "$CANISTER_ID"; then
    success "Canister found in registry"
else
    error "Canister not found in registry"
fi

# Step 3: Get specific canister info
section "Getting specific canister info"
CANISTER_INFO=$(curl -s "$API_URL/canisters/$CANISTER_ID")

if echo "$CANISTER_INFO" | grep -q "$CANISTER_ID"; then
    success "Successfully retrieved canister info"
    echo -e "${BLUE}Canister Info:${NC}"
    echo "$CANISTER_INFO" | grep -v -e "^\[" -e "^\]" | sed 's/^/  /'
else
    error "Failed to get canister info"
fi

# Step 4: Trigger a system refresh to fetch token data
section "Triggering system refresh to fetch token data"
REFRESH_RESPONSE=$(curl -s -X POST "$API_URL/system/refresh")

if echo "$REFRESH_RESPONSE" | grep -q "Refresh triggered successfully"; then
    success "Refresh triggered successfully"
else
    error "Failed to trigger refresh: $REFRESH_RESPONSE"
fi

# Wait for refresh to complete (adjust time as needed)
echo "Waiting for refresh to complete (10 seconds)..."
sleep 10

# Step 5: Check if token info was fetched
section "Checking token info after refresh"
TOKEN_INFO=$(curl -s "$API_URL/tokens/$CANISTER_ID")

if echo "$TOKEN_INFO" | grep -q "$CANISTER_ID"; then
    success "Successfully retrieved token info"
    echo -e "${BLUE}Token Info:${NC}"
    echo "$TOKEN_INFO" | grep -v -e "^\[" -e "^\]" | sed 's/^/  /'
else
    warning "Could not retrieve token info. This could happen if:
    1. The canister is not a valid token canister
    2. The refresh is still in progress
    3. There was an error connecting to the Internet Computer mainnet"
fi

# Step 6: Check if module hash was verified
section "Checking if module hash was verified"
MODULE_HASHES=$(curl -s "$API_URL/module-hashes")

if echo "$MODULE_HASHES" | grep -q "5471eb4e9e70f245d8db1a1673d43ab5ff9443c6d1588f5bdf052bdc7e88f0a5"; then
    success "Found verified module hash in the system"
    echo -e "${BLUE}Verified Module Hashes:${NC}"
    echo "$MODULE_HASHES" | grep -v -e "^\[" -e "^\]" | sed 's/^/  /'
else
    warning "Verified module hash not found. This could happen if:
    1. The background verification is still in progress
    2. The module hash is different than expected"
fi

# Step 7: Get all tokens
section "Getting all tokens"
ALL_TOKENS=$(curl -s "$API_URL/tokens")

echo -e "${BLUE}All Tokens:${NC}"
echo "$ALL_TOKENS" | grep -v -e "^\[" -e "^\]" | sed 's/^/  /'

# Final summary
section "Test Summary"
echo -e "✓ Added canister to registry: $CANISTER_ID"
echo -e "✓ Verified canister exists in registry"
echo -e "✓ Triggered system refresh"
echo -e "✓ Attempted to fetch token info"

echo -e "\n${GREEN}Test completed successfully!${NC}" 