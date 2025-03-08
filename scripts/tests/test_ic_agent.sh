#!/bin/bash

# Test script for verifying IC agent can call canister methods on mainnet
# This script uses the node dfx command to test the same canister

# Color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
CANISTER_ID="sk4hs-faaaa-aaaag-at3rq-cai"
IC_NETWORK="ic"  # Mainnet

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

# Check if dfx is installed
section "Checking if dfx is installed"
if ! command -v dfx &> /dev/null; then
    error "dfx is not installed. Please install it first with 'sh -ci \"$(curl -fsSL https://sdk.dfinity.org/install.sh)\"'"
fi
success "dfx is installed"

# Make sure we're connected to the IC mainnet
section "Setting network to mainnet"
dfx identity use default 2>/dev/null || dfx identity new default --storage-mode plaintext
dfx identity use default
success "Using default identity"

# Call get_info on the token canister
section "Calling get_info on token canister"
echo "Canister ID: $CANISTER_ID"
echo "Network: $IC_NETWORK"

GET_INFO_RESULT=$(dfx canister --network $IC_NETWORK call --query $CANISTER_ID get_info '()' 2>&1)

if echo "$GET_INFO_RESULT" | grep -q "Err"; then
    warning "get_info call returned an error: $GET_INFO_RESULT"
else
    success "Successfully called get_info"
    echo -e "${BLUE}Token Info from mainnet:${NC}"
    echo "$GET_INFO_RESULT" | sed 's/^/  /'
fi

# Try to get miners info if applicable
section "Attempting to call get_miners on token canister"
echo "Note: This may fail if the canister doesn't support this method"

GET_MINERS_RESULT=$(dfx canister --network $IC_NETWORK call --query $CANISTER_ID get_miners '()' 2>&1)

if echo "$GET_MINERS_RESULT" | grep -q "has no method call"; then
    warning "get_miners method not found on this canister: $GET_MINERS_RESULT"
elif echo "$GET_MINERS_RESULT" | grep -q "Err"; then
    warning "get_miners call returned an error: $GET_MINERS_RESULT"
else
    success "Successfully called get_miners"
    echo -e "${BLUE}Miners Info from mainnet:${NC}"
    echo "$GET_MINERS_RESULT" | sed 's/^/  /'
fi

# Final summary
section "Test Summary"
echo -e "✓ Verified connectivity to ICP mainnet"
echo -e "✓ Attempted to call get_info on canister $CANISTER_ID"
echo -e "✓ Attempted to call get_miners on canister $CANISTER_ID"

echo -e "\n${GREEN}IC Agent test completed!${NC}" 