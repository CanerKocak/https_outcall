#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

if [ -f ".server.pid" ]; then
    server_pid=$(cat .server.pid)
    
    if ps -p $server_pid > /dev/null; then
        echo -e "${YELLOW}Stopping server with PID: $server_pid${NC}"
        kill $server_pid
        echo -e "${GREEN}Server stopped${NC}"
    else
        echo -e "${RED}Server process not found. It may have already been stopped.${NC}"
    fi
    
    rm .server.pid
    echo -e "${GREEN}Removed PID file${NC}"
else
    echo -e "${RED}No server PID file found. Server may not be running.${NC}"
fi 