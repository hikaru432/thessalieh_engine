#!/usr/bin/env bash
# Pushes the engine's Docker image to Docker Hub. Run docker-build.sh first.
#
# Log in beforehand (once per machine) with:
#   docker login -u <your-dockerhub-username>
#
# Usage:
#   DOCKERHUB_USERNAME=<your-dockerhub-username> ./scripts/docker-push.sh
#
# Optional:
#   IMAGE_NAME (default: thessalieh_engine)
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

: "${DOCKERHUB_USERNAME:?Set DOCKERHUB_USERNAME to your Docker Hub username}"
IMAGE_NAME="${IMAGE_NAME:-thessalieh_engine}"
GIT_SHA="$(git rev-parse --short HEAD)"
IMAGE="docker.io/${DOCKERHUB_USERNAME}/${IMAGE_NAME}"

docker push "${IMAGE}:${GIT_SHA}"
docker push "${IMAGE}:latest"

echo
echo "Pushed:"
echo "  ${IMAGE}:${GIT_SHA}"
echo "  ${IMAGE}:latest"
