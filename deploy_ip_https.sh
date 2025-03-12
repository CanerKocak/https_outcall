#!/bin/bash

# Script to deploy the server with HTTPS support for an IP address
# Usage: ./deploy_ip_https.sh <server_ip> <load_balancer_ip>

# Check if server IP is provided
if [ -z "$1" ]; then
  echo "Error: Server IP address is required."
  echo "Usage: ./deploy_ip_https.sh <server_ip> <load_balancer_ip>"
  exit 1
fi

# Check if load balancer IP is provided
if [ -z "$2" ]; then
  echo "Error: Load balancer IP address is required."
  echo "Usage: ./deploy_ip_https.sh <server_ip> <load_balancer_ip>"
  exit 1
fi

SERVER_IP=$1
LOAD_BALANCER_IP=$2

# Generate certificates for the load balancer IP
echo "Generating certificates for load balancer IP: $LOAD_BALANCER_IP"
./generate_cert.sh $LOAD_BALANCER_IP

# Build the project
echo "Building the project..."
cargo build --release

# Create remote directories
echo "Creating remote directories..."
ssh root@$SERVER_IP "mkdir -p /root/https_outcall/certs"

# Copy the updated binary, scripts, and certificates
echo "Copying files to server..."
scp target/release/https-outcall root@$SERVER_IP:/root/https_outcall/
scp .env root@$SERVER_IP:/root/https_outcall/
scp https-outcall.service root@$SERVER_IP:/etc/systemd/system/
scp certs/cert.pem certs/key.pem root@$SERVER_IP:/root/https_outcall/certs/

# Update the .env file on the server
echo "Updating .env file on the server..."
ssh root@$SERVER_IP "echo 'USE_HTTPS=true' >> /root/https_outcall/.env"
ssh root@$SERVER_IP "echo 'SSL_CERT_PATH=/root/https_outcall/certs/cert.pem' >> /root/https_outcall/.env"
ssh root@$SERVER_IP "echo 'SSL_KEY_PATH=/root/https_outcall/certs/key.pem' >> /root/https_outcall/.env"

# Reload systemd and restart the service
echo "Reloading systemd and restarting the service..."
ssh root@$SERVER_IP "systemctl daemon-reload && systemctl restart https-outcall.service"

# Check service status
echo "Checking service status..."
ssh root@$SERVER_IP "systemctl status https-outcall.service"

echo "Deployment complete! Your server is now configured for HTTPS."
echo ""
echo "Next steps:"
echo "1. In your DigitalOcean dashboard, go to Networking > Load Balancers"
echo "2. Select your load balancer and go to the Settings tab"
echo "3. In the Forwarding Rules section, add rules to forward:"
echo "   - HTTP (port 80) to HTTP (port 80) on your droplet"
echo "   - HTTPS (port 443) to HTTPS (port 443) on your droplet"
echo "4. In the SSL section, upload the certificates from:"
echo "   - Certificate: certs/cert.pem"
echo "   - Private key: certs/key.pem"
echo ""
echo "To test your setup, run: ./test_https.sh $LOAD_BALANCER_IP" 