# HTTPS Outcall Server with ICP Canister Registry

A server designed to be compatible with Internet Computer canister HTTPS outcalls, featuring a comprehensive registry for tracking and managing ICP canisters, including tokens and miners.

## Purpose

The Internet Computer Protocol requires IPv6 connectivity for its canister HTTPS outcalls. This server is specifically configured to:

1. Bind to IPv6 addresses to ensure compatibility with IC canisters
2. Provide a registry for tracking ICP canisters (tokens, miners, wallets)
3. Automatically update canister information by calling their methods
4. Offer a REST API for managing and querying the registry
5. Run reliably as a system service
6. Support HTTPS for secure communication

## Features

- Binds to IPv6 addresses for compatibility with IC canister HTTPS outcalls
- Persistent SQLite database for storing canister information
- Background job system for periodically updating canister information
- REST API for managing and querying the registry
- Support for different canister types (tokens, miners, wallets)
- Automatically starts on system boot via systemd service
- HTTPS support with TLS/SSL for secure communication
- WebSocket support for real-time updates

## Registry Features

- **Canister Management**: Register, update, and delete canisters
- **Token Tracking**: Automatically fetch and store token information
- **Miner Tracking**: Automatically fetch and store miner information and stats
- **Module Hash Tracking**: Track module hashes for canisters
- **Periodic Updates**: Automatically update canister information every minute
- **Manual Refresh**: Trigger manual updates via API

## HTTPS Configuration

The server supports HTTPS for secure communication. There are several ways to configure HTTPS:

### 1. Self-Signed Certificates (Development Only)

For local development or testing, you can generate self-signed certificates:

```bash
# Generate self-signed certificates
./generate_cert.sh yourdomain.com

# Update .env file
USE_HTTPS=true
LOCAL_DEV=true
SSL_CERT_PATH=certs/cert.pem
SSL_KEY_PATH=certs/key.pem
```

### 2. DigitalOcean Load Balancer (Recommended for Production)

For production, the recommended approach is to use a DigitalOcean Load Balancer with managed SSL certificates:

1. Create a load balancer in the DigitalOcean dashboard
2. Configure your domain's DNS to point to the load balancer's IP address
3. Enable SSL in the load balancer settings (DigitalOcean will handle certificate management)
4. Configure forwarding rules to direct traffic to your server on port 8080
5. Enable "Redirect HTTP to HTTPS" in the load balancer settings

With this setup, you don't need to manage SSL certificates on your server - DigitalOcean handles all SSL termination at the load balancer level.

### 3. Let's Encrypt Certificates (Alternative Production Option)

If you're not using a load balancer, you can use certificates from Let's Encrypt:

```bash
# Set up Let's Encrypt certificates
./setup_letsencrypt.sh yourdomain.com your@email.com

# Update .env file
USE_HTTPS=true
SSL_CERT_PATH=/etc/letsencrypt/live/yourdomain.com/fullchain.pem
SSL_KEY_PATH=/etc/letsencrypt/live/yourdomain.com/privkey.pem
```

The server will automatically detect and use the HTTPS configuration if available.

## Local Development

### Prerequisites

- Rust 1.81 or later
- Basic build tools (build-essential, pkg-config, libssl-dev)

### Setup

Run the setup script to install all dependencies:

```bash
./setup.sh
```

### Building

```bash
cargo build --release
```

### Running Locally

```bash
cargo run --release
```

The server will start on `[::]:8080` (all IPv6 interfaces, port 8080).

## Deployment

### Initial Deployment

To deploy the application to a server:

1. Clone this repository on your server
2. Run the deployment script:

```bash
./deploy.sh
```

This will:
- Install all dependencies
- Build the application
- Configure the firewall
- Set up the systemd service
- Start the application

### Updating the Deployed Application

After making changes to the codebase:

1. Commit and push your changes to the repository
2. On the server, run:

```bash
/root/https_outcall/update_and_restart.sh
```

This will:
- Pull the latest changes
- Rebuild the application
- Restart the service

### Checking Server Status

To verify the server is running correctly:

```bash
./check_server.sh
```

This will test connectivity over both IPv4 and IPv6.

## API Endpoints

### Canister Management

- `GET /canisters`: List all registered canisters
- `POST /canisters`: Register a new canister
- `GET /canisters/{canister_id}`: Get details for a specific canister
- `PUT /canisters/{canister_id}`: Update a canister
- `DELETE /canisters/{canister_id}`: Delete a canister

### Token Management

- `GET /tokens`: List all tokens with their information
- `GET /tokens/{canister_id}`: Get details for a specific token

### Miner Management

- `GET /miners`: List all miners with their information
- `GET /miners/{canister_id}`: Get details for a specific miner
- `GET /miners/{canister_id}/stats`: Get mining stats for a specific miner
- `GET /miners/by-token/{token_canister_id}`: Get miners mining for a specific token

### Module Hash Management

- `GET /module-hashes`: List all module hashes
- `POST /module-hashes`: Add a new module hash

### System Management

- `GET /system/status`: Get system status
- `POST /system/refresh`: Trigger a manual refresh of all canister information

## Storage Implementation

The server uses SQLite for persistent storage:

- **High Performance**: Optimized for fast read and write operations
- **Persistence**: Data is stored on disk and persists across reboots
- **Reliability**: Transactions ensure data integrity
- **Simplicity**: No external database service required

## IPv6 Compatibility

This server is specifically configured to bind to IPv6 addresses to ensure compatibility with Internet Computer canister HTTPS outcalls, which require IPv6 connectivity. Thanks to IPv6 dual-stack compatibility, the server remains accessible via both IPv4 and IPv6 addresses.

## WebSocket Functionality

This application includes a WebSocket server that broadcasts mining events to connected clients. This enables real-time updates for your frontend, creating a FOMO-inducing experience as users can see mining activity as it happens.

### Key Features

1. **Real-time Event Broadcasting**: All miner events (token connections, mining starts, and solutions found) are broadcast to all connected WebSocket clients.

2. **Deduplication Logic**: The server handles duplicate notifications from the Internet Computer, ensuring each event is processed only once.

3. **Visualization**: A test page is included that visualizes mining activity with animations and effects.

### Testing the WebSocket

1. **Start the Server**:
   ```bash
   cargo run
   ```

2. **Open the Test Page**:
   Open your browser and navigate to `http://localhost:8080/test`

3. **Connect to WebSocket**:
   Click the "Connect" button on the test page to establish a WebSocket connection.

4. **Simulate Events**:
   In development mode, you can use the simulation buttons to test different event types.

5. **Real Events**:
   To test with real events, configure your miner canister to send notifications to:
   ```
   http://your-server-ip:8080/miner-notifications
   ```

### Canister Notification API

Miners can send notifications to the `/miner-notifications` endpoint with the following structure:

```json
{
  "event": "solution_found",
  "miner_id": "aaaaa-bbbbb-ccccc-ddddd-eeeee",
  "timestamp": 1646456789000000000,
  "data": {
    "token_id": "aaaaa-bbbbb-ccccc-ddddd-eeeee",
    "block_height": 12345,
    "nonce": 67890,
    "hash": "000000ff00000000000000000000000000000000000000000000000000000000",
    "difficulty": 12345678,
    "reward": 50
  }
}
```

The API requires an `X-API-Key` header for authentication.

### WebSocket Protocol

Clients connect to the WebSocket endpoint at `/ws`. The server sends JSON messages with the following structure:

```json
{
  "event": "solution_found",
  "data": {
    "token_id": "aaaaa-bbbbb-ccccc-ddddd-eeeee",
    "block_height": 12345,
    "nonce": 67890,
    "hash": "000000ff00000000000000000000000000000000000000000000000000000000",
    "difficulty": 12345678,
    "reward": 50
  },
  "timestamp": 1646456789000
}
```

### Handling Duplicate Notifications

The Internet Computer may send the same notification multiple times due to its consensus mechanism. Our server handles this by:

1. Creating a unique cache key for each notification based on miner ID, timestamp, and event type
2. Storing the response in a cache with a 5-minute expiration
3. Returning the cached response for duplicate notifications
4. Only processing and broadcasting the event the first time it's received

This ensures that even if a notification is received multiple times, it will only be broadcast to WebSocket clients once.