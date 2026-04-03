#!/usr/bin/env bash
set -euo pipefail

# Publish 3GS Docker image to GHCR
# Usage: ./scripts/docker-publish.sh [tag]
# Examples:
#   ./scripts/docker-publish.sh           # pushes :latest + :sha-abc1234
#   ./scripts/docker-publish.sh v3.0      # pushes :v3.0 + :latest + :sha-abc1234

IMAGE="ghcr.io/johnzilla/3goodsources"
SHA=$(git rev-parse --short HEAD)
TAG="${1:-}"

# Ensure logged into GHCR
if ! docker manifest inspect "$IMAGE:latest" >/dev/null 2>&1; then
    echo "Logging into GHCR..."
    echo "Run: echo \$GITHUB_TOKEN | docker login ghcr.io -u johnzilla --password-stdin"
    echo "(Create a token at https://github.com/settings/tokens with write:packages scope)"
fi

echo "Building $IMAGE..."
docker build -t "$IMAGE:latest" -t "$IMAGE:sha-$SHA" .

if [ -n "$TAG" ]; then
    docker tag "$IMAGE:latest" "$IMAGE:$TAG"
    echo "Tagged: $IMAGE:$TAG"
fi

echo ""
echo "Pushing to GHCR..."
docker push "$IMAGE:latest"
docker push "$IMAGE:sha-$SHA"
[ -n "$TAG" ] && docker push "$IMAGE:$TAG"

echo ""
echo "Published:"
echo "  $IMAGE:latest"
echo "  $IMAGE:sha-$SHA"
[ -n "$TAG" ] && echo "  $IMAGE:$TAG"
echo ""
echo "Pull: docker pull $IMAGE:latest"
echo "Run:  docker run -p 3000:3000 -e REGISTRY_PATH=/app/registry.json $IMAGE:latest"
