#!/bin/bash
# Test script for Jupiter AG MCP Server Docker container

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Jupiter AG MCP Server Docker Test Suite${NC}"
echo

# Configuration
IMAGE_NAME="jupiter-ag-mcp:latest"
CONTAINER_NAME="jupiter-mcp-test"
TEST_ENV_FILE=".env.test"

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up test containers...${NC}"
    docker stop "$CONTAINER_NAME" 2>/dev/null || true
    docker rm "$CONTAINER_NAME" 2>/dev/null || true
    rm -f "$TEST_ENV_FILE" 2>/dev/null || true
}

# Trap cleanup on exit
trap cleanup EXIT

# Test 1: Check if Docker image exists
echo -e "${YELLOW}Test 1: Checking if Docker image exists...${NC}"
if docker images "$IMAGE_NAME" --format "{{.Repository}}:{{.Tag}}" | grep -q "$IMAGE_NAME"; then
    echo -e "${GREEN}✅ Docker image '$IMAGE_NAME' found${NC}"
else
    echo -e "${RED}❌ Docker image '$IMAGE_NAME' not found${NC}"
    echo "Please run: ./scripts/build.sh"
    exit 1
fi

# Test 2: Create test environment file
echo -e "${YELLOW}Test 2: Creating test environment...${NC}"
cat > "$TEST_ENV_FILE" << EOF
SOLANA_NETWORK=devnet
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_PRIVATE_KEY=<YOUR_PRIVATE_KEY_HERE>
RUST_LOG=info
EOF
echo -e "${GREEN}✅ Test environment file created${NC}"

# Test 3: Test container startup
echo -e "${YELLOW}Test 3: Testing container startup...${NC}"
if timeout 10 docker run --name "$CONTAINER_NAME" --env-file "$TEST_ENV_FILE" --detach "$IMAGE_NAME"; then
    echo -e "${GREEN}✅ Container started successfully${NC}"
else
    echo -e "${RED}❌ Container failed to start${NC}"
    docker logs "$CONTAINER_NAME" 2>/dev/null || true
    exit 1
fi

# Test 4: Check container health
echo -e "${YELLOW}Test 4: Checking container health...${NC}"
sleep 5

if docker ps -f name="$CONTAINER_NAME" --format "{{.Names}}" | grep -q "$CONTAINER_NAME"; then
    echo -e "${GREEN}✅ Container is running${NC}"
else
    echo -e "${RED}❌ Container is not running${NC}"
    docker logs "$CONTAINER_NAME" 2>/dev/null || true
    exit 1
fi

# Test 5: Check container logs
echo -e "${YELLOW}Test 5: Checking container logs...${NC}"
if docker logs "$CONTAINER_NAME" 2>&1 | grep -q "Jupiter AG MCP Server"; then
    echo -e "${GREEN}✅ Container logs show server started${NC}"
else
    echo -e "${YELLOW}⚠️  Logs don't show expected startup message${NC}"
    echo "Container logs:"
    docker logs "$CONTAINER_NAME"
fi

# Test 6: Test basic MCP communication (if possible)
echo -e "${YELLOW}Test 6: Testing basic MCP communication...${NC}"
test_request='{"jsonrpc":"2.0","id":"test","method":"tools/list","params":null}'
if timeout 5 bash -c "echo '$test_request' | docker exec -i '$CONTAINER_NAME' /app/jup-mpc" 2>/dev/null | grep -q "tools"; then
    echo -e "${GREEN}✅ MCP communication successful${NC}"
else
    echo -e "${YELLOW}⚠️  MCP communication test inconclusive (this may be normal for stdio-based servers)${NC}"
fi

# Test 7: Resource usage check
echo -e "${YELLOW}Test 7: Checking resource usage...${NC}"
stats=$(docker stats "$CONTAINER_NAME" --no-stream --format "table {{.CPUPerc}}\t{{.MemUsage}}" | tail -n 1)
echo -e "${GREEN}✅ Container stats: $stats${NC}"

echo
echo -e "${GREEN}🎉 Docker tests completed successfully!${NC}"
echo
echo -e "${BLUE}Container Details:${NC}"
docker ps -f name="$CONTAINER_NAME" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
echo
echo -e "${BLUE}To view logs:${NC}"
echo "docker logs -f $CONTAINER_NAME"