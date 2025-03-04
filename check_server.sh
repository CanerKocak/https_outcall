#!/bin/bash

echo "===== Checking server connectivity ====="

# Check if the server is running
echo "Checking if the service is running..."
sudo systemctl is-active --quiet https-outcall.service
if [ $? -eq 0 ]; then
    echo "✅ Service is running"
else
    echo "❌ Service is not running"
    echo "Starting service..."
    sudo systemctl start https-outcall.service
fi

# Check which ports are being listened on
echo -e "\nChecking listening ports..."
sudo netstat -tuln | grep 8080
if [ $? -eq 0 ]; then
    echo "✅ Server is listening on port 8080"
else
    echo "❌ Server is not listening on port 8080"
fi

# Check IPv4 connectivity
echo -e "\nTesting IPv4 connectivity..."
curl -s -m 5 http://127.0.0.1:8080 > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ IPv4 localhost connectivity works"
else
    echo "❌ IPv4 localhost connectivity failed"
fi

curl -s -m 5 http://24.144.76.120:8080 > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ IPv4 public connectivity works (Reserved IP)"
else
    echo "❌ IPv4 public connectivity failed (Reserved IP)"
fi

curl -s -m 5 http://134.209.193.115:8080 > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ IPv4 public connectivity works (Public IP)"
else
    echo "❌ IPv4 public connectivity failed (Public IP)"
fi

# Check IPv6 connectivity
echo -e "\nTesting IPv6 connectivity..."
curl -6 -s -m 5 http://[::1]:8080 > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ IPv6 localhost connectivity works"
else
    echo "❌ IPv6 localhost connectivity failed"
fi

# Test with the actual IPv6 address
echo -e "\nTesting public IPv6 connectivity..."
curl -6 -s -m 5 http://[2a03:b0c0:2:f0::5786:1]:8080 > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ IPv6 public connectivity works"
    echo "   Your server is accessible at http://[2a03:b0c0:2:f0::5786:1]:8080"
else
    echo "❌ IPv6 public connectivity failed"
    echo "   Troubleshooting:"
    echo "   - Check if the server is binding to IPv6 (netstat -tuln | grep 8080)"
    echo "   - Check if the firewall allows IPv6 traffic (sudo ufw status)"
    echo "   - Check if IPv6 is properly configured (ip -6 addr show)"
fi

# Check firewall status
echo -e "\nChecking firewall status..."
sudo ufw status | grep 8080

echo -e "\n===== Check complete =====" 