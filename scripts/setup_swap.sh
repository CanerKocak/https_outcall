#!/bin/bash

# Script to add swap space to the server
# Usage: ./setup_swap.sh [size_in_GB]

set -e  # Exit on any error

# Default to 2GB if no size specified
SWAP_SIZE=${1:-2}

echo "===== Adding ${SWAP_SIZE}GB swap space ====="

# Check if swap already exists
if grep -q "swapfile" /proc/swaps; then
    echo "Swap already active. Skipping swap creation."
else
    # Create swap file
    echo "Creating ${SWAP_SIZE}GB swap file..."
    fallocate -l ${SWAP_SIZE}G /swapfile
    if [ $? -ne 0 ]; then
        echo "Failed to create swap file with fallocate. Trying dd method..."
        dd if=/dev/zero of=/swapfile bs=1G count=${SWAP_SIZE}
    fi

    # Set permissions
    chmod 600 /swapfile

    # Set up swap area
    mkswap /swapfile
    swapon /swapfile

    # Make swap permanent
    if ! grep -q "swapfile" /etc/fstab; then
        echo '/swapfile none swap sw 0 0' | tee -a /etc/fstab
    fi

    # Adjust swappiness for server environment
    echo "Adjusting swappiness..."
    echo "vm.swappiness=10" | tee -a /etc/sysctl.conf
    sysctl vm.swappiness=10
fi

# Verify swap is active
echo "Swap status:"
swapon --show
free -h 