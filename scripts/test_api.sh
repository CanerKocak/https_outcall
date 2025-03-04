#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Server URL
BASE_URL="http://localhost:8080"

# Helper function to make API calls and check responses
call_api() {
    local method=$1
    local endpoint=$2
    local data=$3
    local description=$4

    echo -e "\n${BLUE}Testing: $description${NC}"
    echo -e "${YELLOW}$method $endpoint${NC}"
    
    if [ -n "$data" ]; then
        echo -e "${YELLOW}Data: $data${NC}"
        response=$(curl -s -X $method "$BASE_URL$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data")
    else
        response=$(curl -s -X $method "$BASE_URL$endpoint")
    fi
    
    if [ $? -eq 0 ] && [ -n "$response" ]; then
        echo -e "${GREEN}Success! Response:${NC}"
        echo $response | jq '.' || echo $response
    else
        echo -e "${RED}Failed! Response:${NC}"
        echo $response
        return 1
    fi
}

# Check if server is running
echo -e "${BLUE}Checking if server is running...${NC}"
if ! curl -s "$BASE_URL/system/status" > /dev/null; then
    echo -e "${RED}Server is not running. Please start the server first with ./scripts/start_server.sh${NC}"
    exit 1
fi

# Test System endpoints
echo -e "\n${YELLOW}=== Testing System Endpoints ===${NC}"
call_api "GET" "/system/status" "" "Get system status"
call_api "POST" "/system/refresh" "" "Trigger system refresh"
call_api "GET" "/system/interfaces" "" "Generate interface files"

# Test Canister endpoints
echo -e "\n${YELLOW}=== Testing Canister Endpoints ===${NC}"
call_api "GET" "/canisters" "" "Get all canisters"

# Register a test canister
CANISTER_DATA='{
    "principal": "test-principal",
    "canister_id": "test-canister-1",
    "canister_type": "token",
    "module_hash": "test-hash"
}'
call_api "POST" "/canisters" "$CANISTER_DATA" "Register new canister"

# Get and update the test canister
call_api "GET" "/canisters/test-canister-1" "" "Get specific canister"

UPDATE_DATA='{
    "principal": "updated-principal",
    "canister_type": "miner"
}'
call_api "PUT" "/canisters/test-canister-1" "$UPDATE_DATA" "Update canister"

# Test Token endpoints
echo -e "\n${YELLOW}=== Testing Token Endpoints ===${NC}"
call_api "GET" "/tokens" "" "Get all tokens"
call_api "GET" "/tokens/test-canister-1" "" "Get specific token"
call_api "DELETE" "/tokens/test-canister-1" "" "Delete token"

# Test Miner endpoints
echo -e "\n${YELLOW}=== Testing Miner Endpoints ===${NC}"
call_api "GET" "/miners" "" "Get all miners"
call_api "GET" "/miners/test-canister-1" "" "Get specific miner"
call_api "GET" "/miners/test-canister-1/stats" "" "Get miner stats"
call_api "GET" "/miners/by-token/test-canister-1" "" "Get miners by token"
call_api "GET" "/miners/stats" "" "Get all mining stats"
call_api "DELETE" "/miners/test-canister-1" "" "Delete miner"

# Clean up - delete test canister
call_api "DELETE" "/canisters/test-canister-1" "" "Delete test canister"

echo -e "\n${GREEN}All API tests completed!${NC}" 