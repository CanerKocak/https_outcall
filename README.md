# HTTPS Outcall Server with ICP Canister Registry

A server designed to be compatible with Internet Computer canister HTTPS outcalls, featuring a comprehensive registry for tracking and managing ICP canisters, including tokens and miners.

## Purpose

The Internet Computer Protocol requires IPv6 connectivity for its canister HTTPS outcalls. This server is specifically configured to:

1. Bind to IPv6 addresses to ensure compatibility with IC canisters
2. Provide a registry for tracking ICP canisters (tokens, miners, wallets)
3. Automatically update canister information by calling their methods
4. Offer a REST API for managing and querying the registry
5. Run reliably as a system service

## Features

- Binds to IPv6 addresses for compatibility with IC canister HTTPS outcalls
- Persistent SQLite database for storing canister information
- Background job system for periodically updating canister information
- REST API for managing and querying the registry
- Support for different canister types (tokens, miners, wallets)
- Automatically starts on system boot via systemd service

## Registry Features

- **Canister Management**: Register, update, and delete canisters
- **Token Tracking**: Automatically fetch and store token information
- **Miner Tracking**: Automatically fetch and store miner information and stats
- **Module Hash Tracking**: Track module hashes for canisters
- **Periodic Updates**: Automatically update canister information every minute
- **Manual Refresh**: Trigger manual updates via API

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