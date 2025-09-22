#!/bin/bash
# Deployment script for Jupiter AG MCP Server

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
IMAGE_NAME="jupiter-ag-mcp"
CONTAINER_NAME="jupiter-mcp-server"
ENV_FILE=".env"

echo -e "${BLUE}🚀 Jupiter AG MCP Server Deployment Script${NC}"
echo

# Check if .env file exists
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${YELLOW}Warning: $ENV_FILE file not found${NC}"
    echo "Creating from template..."
    
    if [ -f ".env.docker" ]; then
        cp .env.docker .env
        echo -e "${YELLOW}Please edit .env file with your actual configuration before running again${NC}"
        exit 1
    else
        echo -e "${RED}Error: No environment template found${NC}"
        exit 1
    fi
fi

# Function to check if container is running
check_container() {
    docker ps -q -f name="$CONTAINER_NAME" 2>/dev/null
}

# Function to check if container exists (running or stopped)
container_exists() {
    docker ps -aq -f name="$CONTAINER_NAME" 2>/dev/null
}

# Stop and remove existing container if it exists
if [ "$(container_exists)" ]; then
    echo -e "${YELLOW}Stopping existing container...${NC}"
    docker stop "$CONTAINER_NAME" 2>/dev/null || true
    docker rm "$CONTAINER_NAME" 2>/dev/null || true
fi

# Build the image
echo -e "${YELLOW}Building Docker image...${NC}"
./scripts/build.sh

# Run the container
echo -e "${YELLOW}Starting container...${NC}"
docker run -d \
    --name "$CONTAINER_NAME" \
    --env-file "$ENV_FILE" \
    --restart unless-stopped \
    --security-opt no-new-privileges:true \
    --read-only \
    --tmpfs /tmp \
    --tmpfs /var/tmp \
    --memory=512m \
    --cpus=0.5 \
    "$IMAGE_NAME:latest"

# Wait a moment for container to start
sleep 3

# Check if container is running
if [ "$(check_container)" ]; then
    echo -e "${GREEN}✅ Container started successfully!${NC}"
    echo
    echo -e "${BLUE}Container Details:${NC}"
    docker ps -f name="$CONTAINER_NAME" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    echo
    echo -e "${BLUE}To view logs:${NC}"
    echo "docker logs -f $CONTAINER_NAME"
    echo
    echo -e "${BLUE}To stop:${NC}"
    echo "docker stop $CONTAINER_NAME"
    echo
    echo -e "${BLUE}To access container:${NC}"
    echo "docker exec -it $CONTAINER_NAME /bin/bash"
else
    echo -e "${RED}❌ Container failed to start${NC}"
    echo "Checking logs..."
    docker logs "$CONTAINER_NAME"
    exit 1
fi