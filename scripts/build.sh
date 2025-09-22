#!/bin/bash
# Build script for Jupiter AG MCP Server Docker image

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
IMAGE_NAME="jupiter-ag-mcp"
TAG="${1:-latest}"
DOCKERFILE="${2:-Dockerfile}"

echo -e "${YELLOW}Building Jupiter AG MCP Server Docker image...${NC}"
echo "Image: ${IMAGE_NAME}:${TAG}"
echo "Dockerfile: ${DOCKERFILE}"
echo

# Check if Dockerfile exists
if [ ! -f "$DOCKERFILE" ]; then
    echo -e "${RED}Error: Dockerfile '$DOCKERFILE' not found${NC}"
    exit 1
fi

# Build the Docker image
echo -e "${YELLOW}Starting Docker build...${NC}"
docker build \
    -t "${IMAGE_NAME}:${TAG}" \
    -f "$DOCKERFILE" \
    --build-arg BUILDKIT_INLINE_CACHE=1 \
    .

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Build completed successfully!${NC}"
    echo "Image: ${IMAGE_NAME}:${TAG}"
    
    # Show image size
    echo -e "${YELLOW}Image size:${NC}"
    docker images "${IMAGE_NAME}:${TAG}" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"
    
    # Tag as latest if not already
    if [ "$TAG" != "latest" ]; then
        docker tag "${IMAGE_NAME}:${TAG}" "${IMAGE_NAME}:latest"
        echo -e "${GREEN}Tagged as ${IMAGE_NAME}:latest${NC}"
    fi
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

echo
echo -e "${YELLOW}To run the container:${NC}"
echo "docker run --env-file .env ${IMAGE_NAME}:${TAG}"
echo
echo -e "${YELLOW}Or use docker-compose:${NC}"
echo "docker-compose up -d"