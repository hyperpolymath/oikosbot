#!/bin/bash
# Eco-Bot Container Build Script
#
# Builds all container images using nerdctl (containerd native CLI)
# Assumes /cerro-torre base image is available

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if nerdctl is available
if ! command -v nerdctl &> /dev/null; then
    log_error "nerdctl not found. Please install nerdctl."
    exit 1
fi

# Check if /cerro-torre base image exists
if ! nerdctl image inspect /cerro-torre &> /dev/null; then
    log_warn "/cerro-torre base image not found locally."
    log_info "Pulling /cerro-torre image..."
    nerdctl pull /cerro-torre || {
        log_error "Failed to pull /cerro-torre image."
        log_info "Please ensure the image is available or build it first."
        exit 1
    }
fi

cd "$PROJECT_ROOT"

# Build main eco-bot image
log_info "Building eco-bot:latest..."
nerdctl build \
    --tag eco-bot:latest \
    --file containers/Containerfile \
    --progress=plain \
    .

# Build policy engine image
log_info "Building eco-bot-policy:latest..."
nerdctl build \
    --tag eco-bot-policy:latest \
    --file containers/Containerfile.policy \
    --progress=plain \
    .

# Tag images
VERSION=$(git describe --tags --always 2>/dev/null || echo "0.1.0")
log_info "Tagging images with version: $VERSION"

nerdctl tag eco-bot:latest "eco-bot:$VERSION"
nerdctl tag eco-bot-policy:latest "eco-bot-policy:$VERSION"

# List built images
log_info "Built images:"
nerdctl images | grep eco-bot

log_info "Build complete!"
echo ""
echo "To run the stack:"
echo "  cd containers && nerdctl compose up -d"
echo ""
echo "To push to registry:"
echo "  nerdctl push eco-bot:$VERSION"
