#!/bin/bash

# Script to deploy SSL certificates to the server
# Usage: ./deploy_certs.sh <server_ip> <domain>

# Check if server IP is provided
if [ -z "$1" ]; then
  echo "Error: Server IP address is required."
  echo "Usage: ./deploy_certs.sh <server_ip> <domain>"
  exit 1
fi

SERVER_IP=$1
DOMAIN=${2:-"localhost"}

# Generate certificates if they don't exist
if [ ! -f "certs/cert.pem" ] || [ ! -f "certs/key.pem" ]; then
  echo "Certificates not found. Generating new certificates..."
  ./generate_cert.sh $DOMAIN
fi

# Create remote directory
echo "Creating remote directory for certificates..."
ssh root@$SERVER_IP "mkdir -p /root/https_outcall/certs"

# Copy certificates to server
echo "Copying certificates to server..."
scp certs/cert.pem certs/key.pem root@$SERVER_IP:/root/https_outcall/certs/

# Check if copy was successful
if [ $? -eq 0 ]; then
  echo "Certificates deployed successfully!"
  
  # Restart the service
  echo "Restarting the service..."
  ssh root@$SERVER_IP "systemctl restart https-outcall.service"
  
  echo "Deployment complete!"
else
  echo "Failed to deploy certificates."
fi 