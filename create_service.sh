#!/bin/bash

# Script to create a systemd service for the https_outcall application
# Usage: ./create_service.sh

# This script should be run on the remote server

# Create systemd service file
cat > /tmp/https_outcall.service << 'EOF'
[Unit]
Description=HTTPS Outcall Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/https_outcall
ExecStart=/root/https_outcall/target/debug/https_outcall
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Move service file to systemd directory
sudo mv /tmp/https_outcall.service /etc/systemd/system/

# Reload systemd, enable and start the service
sudo systemctl daemon-reload
sudo systemctl enable https_outcall
sudo systemctl start https_outcall

echo "Service created and started. Check status with: sudo systemctl status https_outcall" 