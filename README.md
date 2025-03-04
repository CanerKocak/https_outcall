# HTTPS Outcall

A simple Actix Web server that binds to both IPv4 and IPv6 addresses.

## Deployment on Ubuntu 24.10

### Prerequisites

- Ubuntu 24.10 x64
- Internet connection
- Sudo privileges

### Automatic Deployment

1. Clone this repository:
   ```
   git clone https://github.com/yourusername/https_outcall.git
   cd https_outcall
   ```

2. Run the deployment script:
   ```
   ./deploy.sh
   ```

This will:
- Install all dependencies (Rust, Git, etc.)
- Build the application
- Configure the firewall
- Set up and start the systemd service

### Manual Deployment

1. Install dependencies:
   ```
   ./setup.sh
   ```

2. Build the application:
   ```
   cargo build --release
   ```

3. Configure the firewall:
   ```
   sudo ufw allow 8080/tcp
   sudo ufw allow 8080/tcp6
   ```

4. Set up the systemd service:
   ```
   sudo cp https-outcall.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable https-outcall.service
   sudo systemctl start https-outcall.service
   ```

## Usage

Once deployed, the application will be accessible at:
- IPv4: http://24.144.76.120:8080 or http://134.209.193.115:8080
- IPv6: http://[2a03:b0c0:2:f0::5786:1]:8080

Available endpoints:
- `GET /` - Returns "Hello world!"
- `POST /echo` - Echoes back the request body
- `GET /hey` - Returns "Hey there!"

## Service Management

- Check service status:
  ```
  sudo systemctl status https-outcall.service
  ```

- View logs:
  ```
  sudo journalctl -u https-outcall.service -f
  ```

- Restart service:
  ```
  sudo systemctl restart https-outcall.service
  ```

## IPv6 Configuration

IPv6 has been successfully enabled on your DigitalOcean droplet with the following details:

- IPv6 Address: 2a03:b0c0:2:f0::5786:1
- IPv6 Gateway: 2a03:b0c0:2:f0::1
- Address Range: 2a03:b0c0:2:f0::5786:0 - 2a03:b0c0:2:f0::5786:f

## Verifying IPv6 Connectivity

To verify that your server is listening on IPv6:

1. On the server, run:
   ```
   netstat -tuln | grep 8080
   ```
   
   You should see entries for both IPv4 (0.0.0.0:8080) and IPv6 ([::]:8080)

2. Test IPv6 connectivity from the server itself:
   ```
   curl -6 http://[::1]:8080
   ```

3. Test with your public IPv6 address:
   ```
   curl -6 http://[2a03:b0c0:2:f0::5786:1]:8080
   ```

4. From another machine with IPv6 connectivity, try accessing your server:
   ```
   curl -6 http://[2a03:b0c0:2:f0::5786:1]:8080
   ```