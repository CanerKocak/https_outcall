#!/bin/bash

# Exit on error
set -e

echo "===== Fixing UFW Firewall Configuration ====="

# Check UFW version
echo "UFW Version:"
ufw version

# Reset UFW to default settings
echo "Resetting UFW to default settings..."
sudo ufw --force reset

# Set default policies
echo "Setting default policies..."
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Add rules with simple syntax
echo "Adding firewall rules..."
sudo ufw allow 22/tcp  # SSH
sudo ufw allow 8080/tcp  # Our application

# Enable UFW
echo "Enabling UFW..."
sudo ufw --force enable

# Show status
echo "UFW Status:"
sudo ufw status verbose

echo "===== UFW Configuration Fixed ====="

# Now set up the systemd service
echo "===== Setting up systemd service ====="
sudo cp https-outcall.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable https-outcall.service
sudo systemctl restart https-outcall.service

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