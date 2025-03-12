#!/bin/bash

# Script to set up Let's Encrypt certificates for production use
# Usage: ./setup_letsencrypt.sh <domain> <email>

# Check if domain is provided
if [ -z "$1" ]; then
  echo "Error: Domain name is required."
  echo "Usage: ./setup_letsencrypt.sh <domain> <email>"
  exit 1
fi

# Check if email is provided
if [ -z "$2" ]; then
  echo "Error: Email address is required."
  echo "Usage: ./setup_letsencrypt.sh <domain> <email>"
  exit 1
fi

DOMAIN=$1
EMAIL=$2

# Install certbot if not already installed
if ! command -v certbot &> /dev/null; then
  echo "Certbot not found. Installing..."
  apt-get update
  apt-get install -y certbot
fi

# Stop the service if it's running
systemctl stop https-outcall.service

# Get the certificate
echo "Obtaining Let's Encrypt certificate for $DOMAIN..."
certbot certonly --standalone --preferred-challenges http -d $DOMAIN -m $EMAIL --agree-tos --non-interactive

# Check if certificate was obtained successfully
if [ $? -eq 0 ]; then
  echo "Certificate obtained successfully!"
  
  # Create certs directory if it doesn't exist
  mkdir -p /root/https_outcall/certs
  
  # Copy certificates to the application directory
  echo "Copying certificates to application directory..."
  cp /etc/letsencrypt/live/$DOMAIN/fullchain.pem /root/https_outcall/certs/cert.pem
  cp /etc/letsencrypt/live/$DOMAIN/privkey.pem /root/https_outcall/certs/key.pem
  
  # Update environment variables
  echo "Updating environment variables..."
  sed -i "s|SSL_CERT_PATH=.*|SSL_CERT_PATH=/root/https_outcall/certs/cert.pem|" /root/https_outcall/.env
  sed -i "s|SSL_KEY_PATH=.*|SSL_KEY_PATH=/root/https_outcall/certs/key.pem|" /root/https_outcall/.env
  sed -i "s|USE_HTTPS=.*|USE_HTTPS=true|" /root/https_outcall/.env
  
  # Set up auto-renewal
  echo "Setting up auto-renewal..."
  cat > /etc/cron.d/certbot-renew << EOF
0 0,12 * * * root certbot renew --quiet --post-hook "cp /etc/letsencrypt/live/$DOMAIN/fullchain.pem /root/https_outcall/certs/cert.pem && cp /etc/letsencrypt/live/$DOMAIN/privkey.pem /root/https_outcall/certs/key.pem && systemctl restart https-outcall.service"
EOF
  
  # Restart the service
  echo "Restarting the service..."
  systemctl start https-outcall.service
  
  echo "Setup complete! Your server is now using HTTPS with Let's Encrypt certificates."
else
  echo "Failed to obtain certificate."
  
  # Restart the service
  systemctl start https-outcall.service
fi 