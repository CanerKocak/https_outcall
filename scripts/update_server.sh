#!/bin/bash

# Script to update the server with swap space
# Usage: ./update_server.sh

set -e  # Exit on any error

# Server details
SERVER_IP="134.209.193.115"
SSH_KEY="~/.ssh/id_ed25519"

echo "===== Step 1: Committing and pushing local changes ====="
git add .
git commit -m "Update server with swap space setup" || echo "No changes to commit"
git push origin main || echo "No changes to push"

echo "===== Step 2: Setting up swap space on server ====="
# Upload the swap setup script
scp -i $SSH_KEY scripts/setup_swap.sh root@$SERVER_IP:~/https_outcall/scripts/
# Make it executable and run it
ssh -i $SSH_KEY root@$SERVER_IP "cd ~/https_outcall/scripts && chmod +x setup_swap.sh && ./setup_swap.sh 4"

echo "===== Step 3: Updating and building the server ====="
ssh -i $SSH_KEY root@$SERVER_IP "cd ~/https_outcall && git reset --hard && git fetch origin && git checkout origin/main -B main && source ~/.cargo/env && RUSTFLAGS=\"-C link-arg=-Wl,--no-keep-memory -C link-arg=-Wl,--reduce-memory-overheads\" cargo build --release && sudo systemctl restart https-outcall.service"

echo "===== Server update complete ====="
echo "Checking server status..."
ssh -i $SSH_KEY root@$SERVER_IP "systemctl status https-outcall.service" 