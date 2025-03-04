# HTTPS Outcall Server

A simple Actix Web server designed to be compatible with Internet Computer canister HTTPS outcalls.

## Features

- Binds to IPv6 addresses for compatibility with IC canister HTTPS outcalls
- Provides a simple API with GET and POST endpoints
- Automatically starts on system boot via systemd service

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
/root/update_and_restart.sh
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

- `GET /`: Returns "Hello world!"
- `POST /echo`: Echoes back the request body
- `GET /hey`: Returns "Hey there!"

## IPv6 Compatibility

This server is specifically configured to bind to IPv6 addresses to ensure compatibility with Internet Computer canister HTTPS outcalls, which require IPv6 connectivity. Thanks to IPv6 dual-stack compatibility, the server remains accessible via both IPv4 and IPv6 addresses.