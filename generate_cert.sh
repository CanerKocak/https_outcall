#!/bin/bash

# Script to generate self-signed SSL certificates for development
# For production, you should use certificates from a trusted CA

# Set variables
CERT_DIR="certs"
CERT_FILE="$CERT_DIR/cert.pem"
KEY_FILE="$CERT_DIR/key.pem"
IP_ADDRESS=${1:-"127.0.0.1"}

# Create directory if it doesn't exist
mkdir -p $CERT_DIR

echo "Generating self-signed certificate for IP address: $IP_ADDRESS"

# Generate private key and certificate
openssl req -x509 -newkey rsa:4096 -nodes -keyout $KEY_FILE -out $CERT_FILE -days 365 \
  -subj "/CN=$IP_ADDRESS" \
  -addext "subjectAltName = IP:$IP_ADDRESS"

# Check if generation was successful
if [ $? -eq 0 ]; then
  echo "Certificate generated successfully!"
  echo "Certificate: $CERT_FILE"
  echo "Private key: $KEY_FILE"
  
  # Make the script executable
  chmod +x $0
  
  echo ""
  echo "For production use with a load balancer:"
  echo "1. Upload these certificates to your DigitalOcean load balancer"
  echo "2. Configure your load balancer to forward ports 80 and 443 to your server"
  echo "3. Make sure your server is using HTTPS (USE_HTTPS=true in .env)"
else
  echo "Failed to generate certificate."
fi 