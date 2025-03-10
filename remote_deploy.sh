#!/bin/bash

# Script to deploy and run the application on a remote server
# Usage: ./remote_deploy.sh [server_address] [username] [swap_size_in_GB] [create_service]

# Default values
SERVER=${1:-"ubuntu-s-1vcpu-1gb-ams3-01"}
USERNAME=${2:-"root"}
SWAP_SIZE=${3:-2}
CREATE_SERVICE=${4:-"yes"}  # Default to creating a service

# Check if SSH key exists, if not, use password authentication
SSH_KEY="$HOME/.ssh/id_rsa"
if [ -f "$SSH_KEY" ]; then
    SSH_CMD="ssh -i $SSH_KEY"
    SCP_CMD="scp -i $SSH_KEY"
else
    SSH_CMD="ssh"
    SCP_CMD="scp"
fi

echo "===== Deploying to $USERNAME@$SERVER ====="

# Upload the setup script
echo "Uploading setup script..."
$SCP_CMD setup_and_run.sh $USERNAME@$SERVER:~/https_outcall/scripts/

# Upload the service creation script if needed
if [ "$CREATE_SERVICE" = "yes" ]; then
    echo "Uploading service creation script..."
    $SCP_CMD create_service.sh $USERNAME@$SERVER:~/https_outcall/scripts/
fi

# Make the script executable and run it
echo "Running setup script on remote server..."
$SSH_CMD $USERNAME@$SERVER "cd ~/https_outcall/scripts && chmod +x setup_and_run.sh && ./setup_and_run.sh $SWAP_SIZE"

# Create and start the service if requested
if [ "$CREATE_SERVICE" = "yes" ]; then
    echo "Setting up systemd service..."
    $SSH_CMD $USERNAME@$SERVER "cd ~/https_outcall/scripts && chmod +x create_service.sh && ./create_service.sh"
fi

echo "===== Deployment complete ====="
echo "If the server started successfully, it should now be running on the remote machine."

if [ "$CREATE_SERVICE" = "yes" ]; then
    echo "The application is running as a systemd service. Check status with:"
    echo "  $SSH_CMD $USERNAME@$SERVER 'systemctl status https_outcall'"
else
    echo "To check the server status, SSH into the server and use:"
    echo "  $SSH_CMD $USERNAME@$SERVER 'ps aux | grep https_outcall'"
fi 