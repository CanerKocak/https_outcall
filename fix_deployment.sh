#!/bin/bash

# Exit on error
set -e

echo "===== Fixing deployment issues for https_outcall application ====="

# Ensure scripts are executable
chmod +x deploy.sh
chmod +x setup.sh
chmod +x check_server.sh

# Fix UFW configuration
echo "Fixing firewall configuration..."
sudo ufw allow 8080/tcp comment 'Allow HTTP traffic for https_outcall (IPv4)'
sudo ufw allow 8080/tcp6 comment 'Allow HTTP traffic for https_outcall (IPv6)'
sudo ufw status verbose

# Build the application
echo "Building application..."
. "$HOME/.cargo/env"
cargo build --release

# Set up systemd service
echo "Setting up systemd service..."
sudo cp https-outcall.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable https-outcall.service
sudo systemctl restart https-outcall.service
sudo systemctl status https-outcall.service || true

# Check if the service is running
echo "Checking if the service is running..."
if sudo systemctl is-active --quiet https-outcall.service; then
    echo "Service is running successfully!"
else
    echo "Service failed to start. Checking logs..."
    sudo journalctl -u https-outcall.service -n 50
fi

echo "===== Deployment fixes complete! ====="
echo "Your application should now be accessible at:"
echo "  - http://24.144.76.120:8080 (Reserved IPv4)"
echo "  - http://134.209.193.115:8080 (Public IPv4)"
echo "  - http://[2a03:b0c0:2:f0::5786:1]:8080 (IPv6)"
echo ""
echo "To check the service status: sudo systemctl status https-outcall.service"
echo "To view logs: sudo journalctl -u https-outcall.service -f"
echo ""
echo "To verify connectivity: ./check_server.sh" 