#!/bin/bash

# Exit on error
set -e

echo "===== Deploying https_outcall application ====="

# Install dependencies if not already done
if [ ! -f "/etc/systemd/system/https-outcall.service" ]; then
    echo "Running setup script..."
    ./setup.sh
fi

# Source Rust environment
echo "Setting up Rust environment..."
. "$HOME/.cargo/env"

# Build the application
echo "Building application..."
cargo build --release

# Set up firewall
echo "Configuring firewall..."
sudo ufw allow 8080/tcp
sudo ufw allow 8080/tcp6
sudo ufw status

# Copy systemd service file
echo "Setting up systemd service..."
sudo cp https-outcall.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable https-outcall.service
sudo systemctl restart https-outcall.service
sudo systemctl status https-outcall.service

echo "===== Deployment complete! ====="
echo "Your application should now be accessible at:"
echo "  - http://24.144.76.120:8080 (Reserved IPv4)"
echo "  - http://134.209.193.115:8080 (Public IPv4)"
echo "  - http://[2a03:b0c0:2:f0::5786:1]:8080 (IPv6)"
echo ""
echo "To check the service status: sudo systemctl status https-outcall.service"
echo "To view logs: sudo journalctl -u https-outcall.service -f"
echo ""
echo "To verify connectivity: ./check_server.sh" 