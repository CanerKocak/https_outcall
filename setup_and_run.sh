#!/bin/bash

# Script to add swap space, build the project, and run the server
# Usage: ./setup_and_run.sh [swap_size_in_GB]

set -e  # Exit on any error

# Default to 2GB if no size specified
SWAP_SIZE=${1:-2}

echo "===== STEP 1: Adding ${SWAP_SIZE}GB swap space ====="

# Check if swap already exists
if grep -q "swapfile" /proc/swaps; then
    echo "Swap already active. Skipping swap creation."
else
    # Create swap file
    echo "Creating ${SWAP_SIZE}GB swap file..."
    sudo fallocate -l ${SWAP_SIZE}G /swapfile
    if [ $? -ne 0 ]; then
        echo "Failed to create swap file with fallocate. Trying dd method..."
        sudo dd if=/dev/zero of=/swapfile bs=1G count=${SWAP_SIZE}
    fi

    # Set permissions
    sudo chmod 600 /swapfile

    # Set up swap area
    sudo mkswap /swapfile
    sudo swapon /swapfile

    # Make swap permanent
    if ! grep -q "swapfile" /etc/fstab; then
        echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
    fi

    # Adjust swappiness for server environment
    echo "Adjusting swappiness..."
    echo "vm.swappiness=10" | sudo tee -a /etc/sysctl.conf
    sudo sysctl vm.swappiness=10
fi

# Verify swap is active
echo "Swap status:"
sudo swapon --show
free -h

echo "===== STEP 2: Building the project with memory optimizations ====="

# Navigate to project root
cd "$(dirname "$0")/.."

# Build with memory optimizations
echo "Building project with memory optimization flags..."
RUSTFLAGS="-C link-arg=-Wl,--no-keep-memory -C link-arg=-Wl,--reduce-memory-overheads" cargo build

echo "===== STEP 3: Starting the server ====="

# Run the server
echo "Starting the server..."
./target/debug/https_outcall

# Note: If your server needs specific arguments, add them after the binary name above 