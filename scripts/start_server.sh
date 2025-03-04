#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Building project...${NC}"
cargo build

echo -e "${YELLOW}Starting server in the background...${NC}"
# Kill any existing server process
if [ -f ".server.pid" ]; then
    old_pid=$(cat .server.pid)
    if ps -p $old_pid > /dev/null; then
        echo -e "${YELLOW}Stopping existing server (PID: $old_pid)${NC}"
        kill $old_pid
    fi
    rm .server.pid
fi

# Start the server and store its PID
cargo run -- --host 127.0.0.1 --port 8080 > server.log 2>&1 &
server_pid=$!
echo $server_pid > .server.pid

echo -e "${GREEN}Server started with PID: $server_pid${NC}"
echo -e "${GREEN}Server output is being logged to server.log${NC}"
echo -e "${GREEN}To stop the server, run ./scripts/stop_server.sh${NC}"

# Wait for server to start
echo -e "${YELLOW}Waiting for server to start...${NC}"
sleep 3

# Check if server is running by querying the status endpoint
response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/system/status || echo "failed")

if [ "$response" == "200" ]; then
    echo -e "${GREEN}Server is up and running!${NC}"
else
    echo -e "${YELLOW}Server might not be running properly. Check server.log for details.${NC}"
    echo -e "${YELLOW}Last 10 lines of server.log:${NC}"
    tail -n 10 server.log
fi 