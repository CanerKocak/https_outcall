#!/bin/bash

# Master test script for ICP Canister Registry
# This script runs all test cases for the canister registry

# Color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Function to print colored section headers
section() {
    echo -e "\n${BLUE}=======================================================${NC}"
    echo -e "${BLUE}== $1 ==${NC}"
    echo -e "${BLUE}=======================================================${NC}"
}

# Function to print error messages and exit
error() {
    echo -e "${RED}✗ $1${NC}"
    exit 1
}

# Check if all test scripts exist
if [ ! -f "${SCRIPT_DIR}/test_add_canister.sh" ] || [ ! -f "${SCRIPT_DIR}/test_ic_agent.sh" ]; then
    error "Test scripts not found. Make sure test_add_canister.sh and test_ic_agent.sh exist."
fi

# Ensure all test scripts are executable
chmod +x "${SCRIPT_DIR}/test_add_canister.sh"
chmod +x "${SCRIPT_DIR}/test_ic_agent.sh"

# Display testing plan
section "ICP Canister Registry Test Plan"
echo -e "This test plan verifies the following:"
echo -e ""
echo -e "1. ${YELLOW}API Functionality Test${NC}"
echo -e "   - Registering a canister in the registry"
echo -e "   - Verifying the canister is properly added to the database"
echo -e "   - Triggering the refresh mechanism to fetch updated data"
echo -e "   - Verifying token information can be retrieved"
echo -e ""
echo -e "2. ${YELLOW}IC Agent Connectivity Test${NC}"
echo -e "   - Verifying direct connectivity to the Internet Computer mainnet"
echo -e "   - Testing canister method calls (get_info, get_miners) on mainnet"
echo -e ""
echo -e "Together, these tests verify the complete functionality of the registry system"
echo -e "from API interaction to data retrieval from the Internet Computer."

# Prompt user to start tests
echo -e ""
read -p "Press Enter to begin testing..." val

# Run API test
section "Running API Functionality Test"
"${SCRIPT_DIR}/test_add_canister.sh"
API_TEST_RESULT=$?

# Run IC Agent test
section "Running IC Agent Connectivity Test"
"${SCRIPT_DIR}/test_ic_agent.sh"
IC_AGENT_TEST_RESULT=$?

# Display final results
section "Test Results Summary"

if [ $API_TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}✓ API Functionality Test: PASSED${NC}"
else
    echo -e "${RED}✗ API Functionality Test: FAILED${NC}"
fi

if [ $IC_AGENT_TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}✓ IC Agent Connectivity Test: PASSED${NC}"
else
    echo -e "${RED}✗ IC Agent Connectivity Test: FAILED${NC}"
fi

echo -e ""
if [ $API_TEST_RESULT -eq 0 ] && [ $IC_AGENT_TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}All tests completed successfully!${NC}"
    echo -e ""
    echo -e "Your ICP Canister Registry is correctly configured and can:"
    echo -e "1. Register and track canisters via the API"
    echo -e "2. Connect to the Internet Computer mainnet"
    echo -e "3. Retrieve up-to-date information from registered canisters"
    echo -e ""
    echo -e "The system is ready to track hundreds of canisters as required."
    exit 0
else
    echo -e "${RED}Some tests failed. Please review the output above for details.${NC}"
    exit 1
fi 