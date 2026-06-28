#!/bin/bash
# Deploy script — run this on your VPS
# Usage: ./deploy.sh <github-username> <repo-name>

set -e

USERNAME="${1:-your-github-username}"
REPO="${2:-turkish-times}"
IMAGE="ghcr.io/$USERNAME/$REPO:latest"

echo "==> Pulling latest image..."
docker pull $IMAGE

echo "==> Stopping old container..."
docker stop turkish-times 2>/dev/null || true
docker rm turkish-times 2>/dev/null || true

echo "==> Starting new container..."
docker run -d \
  --name turkish-times \
  --restart unless-stopped \
  -p 8000:8000 \
  -e APP_URL="http://your-domain.com" \
  -e SESSION_SECRET="$(openssl rand -hex 32)" \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/uploads:/app/public/uploads \
  $IMAGE

echo "==> Done! Container running on port 8000"
docker logs turkish-times --tail 5
