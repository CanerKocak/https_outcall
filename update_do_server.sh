#!/bin/bash

# Script to update the DigitalOcean server with the new HTTPS configuration
# Usage: ./update_do_server.sh <server_ip>

# Check if server IP is provided
if [ -z "$1" ]; then
  echo "Error: Server IP address is required."
  echo "Usage: ./update_do_server.sh <server_ip>"
  exit 1
fi

SERVER_IP=$1

# Build the project
echo "Building the project..."
cargo build --release

# Create remote directories
echo "Creating remote directories..."
ssh root@$SERVER_IP "mkdir -p /root/https_outcall/certs"

# Copy the updated binary and scripts
echo "Copying files to server..."
scp target/release/https-outcall root@$SERVER_IP:/root/https_outcall/
scp .env root@$SERVER_IP:/root/https_outcall/
scp https-outcall.service root@$SERVER_IP:/etc/systemd/system/
scp generate_cert.sh setup_letsencrypt.sh root@$SERVER_IP:/root/https_outcall/

# Make scripts executable on the server
echo "Making scripts executable..."
ssh root@$SERVER_IP "chmod +x /root/https_outcall/generate_cert.sh /root/https_outcall/setup_letsencrypt.sh"

# Generate self-signed certificates for initial setup
echo "Generating self-signed certificates..."
ssh root@$SERVER_IP "cd /root/https_outcall && ./generate_cert.sh $SERVER_IP"

# Reload systemd and restart the service
echo "Reloading systemd and restarting the service..."
ssh root@$SERVER_IP "systemctl daemon-reload && systemctl restart https-outcall.service"

# Check service status
echo "Checking service status..."
ssh root@$SERVER_IP "systemctl status https-outcall.service"

echo "Update complete! Your server is now configured for HTTPS."
echo ""
echo "Next steps:"
echo "1. Configure your DigitalOcean Load Balancer to forward ports 80 and 443 to your server"
echo "2. For production, run: ssh root@$SERVER_IP 'cd /root/https_outcall && ./setup_letsencrypt.sh yourdomain.com your@email.com'" 