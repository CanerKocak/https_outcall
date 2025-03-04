#!/bin/bash

# Exit on error
set -e

echo "===== Setting up development environment for https_outcall project ====="

# Update package lists
echo "Updating package lists..."
sudo apt-get update

# Install basic dependencies
echo "Installing basic dependencies..."
sudo apt-get install -y \
    build-essential \
    curl \
    wget \
    pkg-config \
    libssl-dev \
    git

# Install Rust (if not already installed)
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # Source the environment directly instead of using source command
    . "$HOME/.cargo/env"
else
    echo "Rust is already installed. Updating..."
    . "$HOME/.cargo/env"
    rustup update
fi

# Install specific Rust version (1.81)
echo "Installing Rust 1.81..."
. "$HOME/.cargo/env"
rustup install 1.81
rustup default 1.81

# Verify installations
echo "Verifying installations..."
git --version
rustc --version
cargo --version

echo "===== Setup complete! ====="
echo "You can now build the project with: cargo build"
echo ""
echo "To switch between Rust versions:"
echo "  - Use 'rustup default 1.81' for version 1.81"
echo "  - Use 'rustup default 1.80.1' for version 1.80.1 (if needed)"
echo "  - Use 'rustup default stable' for the latest stable version"
echo ""
echo "Note: This project requires Rust 1.81 or later due to dependency requirements." 