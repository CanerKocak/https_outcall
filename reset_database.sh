#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Resetting database on Digital Ocean server...${NC}"

# Step 1: Stop the server
echo -e "${YELLOW}Stopping the server...${NC}"
./scripts/stop_server.sh

# Step 2: Backup the current database
echo -e "${YELLOW}Backing up current database...${NC}"
if [ -f "data/registry.db" ]; then
    # Get timestamp for unique backup name
    TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
    mv data/registry.db "data/registry_backup_${TIMESTAMP}.db"
    echo -e "${GREEN}Current database backed up to data/registry_backup_${TIMESTAMP}.db${NC}"
else
    echo -e "${RED}No database file found at data/registry.db${NC}"
    echo -e "${YELLOW}Will create a new database when server starts${NC}"
fi

# Step 3: Start the server (which will create a new database)
echo -e "${YELLOW}Starting the server with a fresh database...${NC}"
./scripts/start_server.sh

echo -e "${GREEN}Database reset complete!${NC}"
echo -e "${GREEN}A new empty database has been created.${NC}"
echo -e "${YELLOW}Your previous database has been backed up with a timestamp.${NC}" 