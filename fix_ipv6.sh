#!/bin/bash

# Exit on error
set -e

echo "===== Fixing IPv6 configuration and server issues ====="

# Step 1: Update netplan configuration to add IPv6 address
echo "Updating netplan configuration..."
NETPLAN_FILE=$(ls /etc/netplan/*.yaml | head -n 1)
echo "Found netplan file: $NETPLAN_FILE"

# Backup the original file
sudo cp $NETPLAN_FILE ${NETPLAN_FILE}.bak
echo "Backed up original netplan file to ${NETPLAN_FILE}.bak"

# Create a new netplan configuration file with IPv6 support
cat > /tmp/50-cloud-init.yaml << 'EOF'
network:
  version: 2
  ethernets:
    eth0:
      match:
        macaddress: "1a:f1:df:18:7c:cf"
      addresses:
      - "134.209.193.115/20"
      - "10.18.0.5/16"
      - "2a03:b0c0:2:f0::5786:1/64"
      nameservers:
        addresses:
        - 67.207.67.2
        - 67.207.67.3
        search: []
      set-name: "eth0"
      mtu: 1500
      routes:
      - to: "0.0.0.0/0"
        via: "134.209.192.1"
      - to: "::/0"
        via: "2a03:b0c0:2:f0::1"
    eth1:
      match:
        macaddress: "ca:6d:14:56:31:9a"
      addresses:
      - "10.110.0.2/20"
      nameservers:
        addresses:
        - 67.207.67.2
        - 67.207.67.3
        search: []
      set-name: "eth1"
      mtu: 1500
EOF

sudo cp /tmp/50-cloud-init.yaml $NETPLAN_FILE
echo "Updated netplan configuration:"
cat $NETPLAN_FILE

# Apply the netplan configuration
echo "Applying netplan configuration..."
sudo netplan apply

# Step 2: Kill any processes using port 8081 and reset the socket
echo "Killing any processes using port 8081..."
sudo fuser -k 8081/tcp || true
sudo fuser -k 8081/tcp6 || true

echo "Resetting socket..."
sudo ss -K '( sport = :8081 )' || true
sudo ss -K '( dport = :8081 )' || true

# Step 3: Update systemd service file
echo "Updating systemd service file..."
cat > /tmp/https-outcall.service << 'EOF'
[Unit]
Description=HTTPS Outcall Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/https_outcall
ExecStart=/root/https_outcall/target/release/https_outcall
Restart=on-failure
RestartSec=5
StartLimitBurst=3
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

sudo cp /tmp/https-outcall.service /etc/systemd/system/https-outcall.service
sudo systemctl daemon-reload

# Step 4: Reset and configure firewall
echo "Resetting and configuring firewall..."
sudo ufw --force reset
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 22/tcp comment 'Allow SSH traffic'
sudo ufw allow 8081/tcp comment 'Allow HTTP traffic for https_outcall'
sudo ufw --force enable

# Step 5: Restart the service
echo "Stopping the service..."
sudo systemctl stop https-outcall.service || true

echo "Waiting for port to be released..."
sleep 5

echo "Starting the service..."
sudo systemctl start https-outcall.service

# Step 6: Verify the service is running
echo "Verifying the service is running..."
if sudo systemctl is-active --quiet https-outcall.service; then
    echo "✅ Service is running successfully!"
else
    echo "❌ Service failed to start. Checking logs..."
    sudo journalctl -u https-outcall.service -n 50
fi

# Step 7: Verify IPv6 configuration
echo "Verifying IPv6 configuration..."
ip -6 addr show
ip -6 route show

# Step 8: Test connectivity
echo "Testing connectivity..."
echo "IPv4 localhost:"
curl -s -m 5 http://127.0.0.1:8081 || echo "Failed to connect to IPv4 localhost"
echo "IPv6 localhost:"
curl -6 -s -m 5 http://[::1]:8081 || echo "Failed to connect to IPv6 localhost"
echo "IPv4 public:"
curl -s -m 5 http://134.209.193.115:8081 || echo "Failed to connect to IPv4 public"
echo "IPv6 public:"
curl -6 -s -m 5 http://[2a03:b0c0:2:f0::5786:1]:8081 || echo "Failed to connect to IPv6 public"

echo "===== Fix complete! ====="
echo "Your application should now be accessible at:"
echo "  - http://134.209.193.115:8081 (IPv4)"
echo "  - http://[2a03:b0c0:2:f0::5786:1]:8081 (IPv6)"
echo ""
echo "To check the service status: sudo systemctl status https-outcall.service"
echo "To view logs: sudo journalctl -u https-outcall.service -f" 