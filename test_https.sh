#!/bin/bash

# Script to test if the server is running with HTTPS
# Usage: ./test_https.sh <load_balancer_ip>

# Check if load balancer IP is provided
if [ -z "$1" ]; then
  echo "Error: Load balancer IP address is required."
  echo "Usage: ./test_https.sh <load_balancer_ip>"
  exit 1
fi

IP=$1

echo "Testing connectivity to load balancer at $IP..."
ping -c 1 $IP

echo ""
echo "-----------------------------------"
echo ""

# Test HTTP connection
echo "Testing HTTP connection..."
curl -v -k http://$IP/ws-status

echo ""
echo "-----------------------------------"
echo ""

# Test HTTPS connection
echo "Testing HTTPS connection..."
curl -v -k https://$IP/ws-status

echo ""
echo "-----------------------------------"
echo ""

# Test WebSocket connection
echo "Testing WebSocket connection..."
echo "Note: This test will fail if the server is not running with HTTPS."
echo "You can test WebSocket connection manually by visiting:"
echo "http://$IP/test or https://$IP/test (accept the self-signed certificate warning)" 