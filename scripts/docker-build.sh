#!/usr/bin/env bash
# Builds the engine's Docker image, tagged for Docker Hub.
#
# Usage:
#   DOCKERHUB_USERNAME=<your-dockerhub-username> ./scripts/docker-build.sh
#
# Optional:
#   IMAGE_NAME (default: thessalieh_engine)
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

: "${DOCKERHUB_USERNAME:?Set DOCKERHUB_USERNAME to your Docker Hub username}"
IMAGE_NAME="${IMAGE_NAME:-thessalieh_engine}"
GIT_SHA="$(git rev-parse --short HEAD)"
IMAGE="docker.io/${DOCKERHUB_USERNAME}/${IMAGE_NAME}"

echo "Building ${IMAGE}:${GIT_SHA} and ${IMAGE}:latest"
docker build -t "${IMAGE}:${GIT_SHA}" -t "${IMAGE}:latest" .

echo
echo "Built:"
echo "  ${IMAGE}:${GIT_SHA}"
echo "  ${IMAGE}:latest"
