#!/bin/bash

# Script to deploy the server to DigitalOcean using git
# Usage: ./deploy_ip_https.sh <server_ip> <load_balancer_ip>

# Check if server IP is provided
if [ -z "$1" ]; then
  echo "Error: Server IP address is required."
  echo "Usage: ./deploy_ip_https.sh <server_ip> <load_balancer_ip>"
  exit 1
fi

SERVER_IP=$1
LOAD_BALANCER_IP=${2:-""}

# Build the project locally
echo "Building the project locally..."
cargo build --release

# Copy the binary to the server
echo "Copying binary to the server..."
scp target/release/https-outcall root@$SERVER_IP:/root/https_outcall/target/release/

# Push changes to the remote repository
echo "Pushing changes to the remote repository..."
git push

# SSH into the server and pull the latest changes
echo "Pulling latest changes on the server..."
ssh root@$SERVER_IP "cd /root/https_outcall && git pull"

# Reload systemd and restart the service
echo "Reloading systemd and restarting the service..."
ssh root@$SERVER_IP "systemctl daemon-reload && systemctl restart https-outcall.service"

# Check service status
echo "Checking service status..."
ssh root@$SERVER_IP "systemctl status https-outcall.service"

echo "Deployment complete! Your server is now running."
echo ""
echo "Note: SSL is now handled by your DigitalOcean load balancer."
echo "Make sure your load balancer is properly configured to forward traffic to your server."
echo "" 