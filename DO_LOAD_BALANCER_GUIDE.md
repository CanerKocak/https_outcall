# DigitalOcean Load Balancer Configuration Guide

This guide will help you configure your DigitalOcean load balancer to work with your HTTPS-enabled server using just an IP address (no domain name).

## Prerequisites

1. A DigitalOcean account
2. A Droplet running your server
3. Self-signed certificates generated with `./generate_cert.sh <load_balancer_ip>`

## Step 1: Create a Load Balancer

1. Log in to your DigitalOcean account
2. Go to **Networking** > **Load Balancers**
3. Click **Create Load Balancer**
4. Choose a datacenter region (same as your Droplet)
5. Under **Choose Droplets**, select your server Droplet
6. Click **Create Load Balancer**

## Step 2: Configure Forwarding Rules

1. Go to your load balancer's detail page
2. Click the **Settings** tab
3. In the **Forwarding Rules** section, click **Edit**
4. Configure the following rules:
   - HTTP on port 80 to HTTP on port 80
   - HTTPS on port 443 to HTTPS on port 443
5. Click **Save**

## Step 3: Configure SSL

1. In the **Settings** tab, scroll to the **SSL** section
2. Click **Add SSL Certificate**
3. Choose **Upload a Certificate**
4. Enter a name for your certificate (e.g., "Server SSL")
5. Upload your certificate files:
   - **Certificate**: `certs/cert.pem`
   - **Private Key**: `certs/key.pem`
6. Click **Save SSL Certificate**

## Step 4: Test Your Configuration

1. Run the test script: `./test_https.sh <load_balancer_ip>`
2. Verify that both HTTP and HTTPS connections work
3. Visit `http://<load_balancer_ip>/test` or `https://<load_balancer_ip>/test` in your browser
   - Note: You'll need to accept the self-signed certificate warning in your browser

## Troubleshooting

### Certificate Issues

If you see certificate errors:
1. Make sure the certificate was generated for the load balancer's IP address
2. Regenerate the certificate: `./generate_cert.sh <load_balancer_ip>`
3. Re-upload the certificate to the load balancer

### Connection Issues

If you can't connect to the load balancer:
1. Check that your Droplet is running
2. Verify that the load balancer's health check is passing
3. Make sure your server is listening on both port 80 and 443
4. Check your server logs: `ssh root@<server_ip> "journalctl -u https-outcall.service -f"`

### HTTPS Not Working

If HTTP works but HTTPS doesn't:
1. Verify that your server has `USE_HTTPS=true` in the `.env` file
2. Check that the certificate paths are correct in the `.env` file
3. Restart your server: `ssh root@<server_ip> "systemctl restart https-outcall.service"`
4. Check the server logs for TLS errors 