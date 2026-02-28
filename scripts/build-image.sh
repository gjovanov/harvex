#!/bin/bash
set -euo pipefail

# Build Docker image, export, transfer to worker2, and import into containerd
IMAGE_NAME="harvex"
IMAGE_TAG="latest"
WORKER="k8s-worker2"

echo "==> Building Docker image ${IMAGE_NAME}:${IMAGE_TAG}..."
cd "$(dirname "$0")/.."
docker build -t "${IMAGE_NAME}:${IMAGE_TAG}" .

echo "==> Exporting image to tarball..."
docker save "${IMAGE_NAME}:${IMAGE_TAG}" -o "/tmp/${IMAGE_NAME}.tar"

echo "==> Transferring to ${WORKER}..."
scp "/tmp/${IMAGE_NAME}.tar" "${WORKER}:/tmp/${IMAGE_NAME}.tar"

echo "==> Importing into containerd on ${WORKER}..."
ssh "${WORKER}" "sudo ctr -n k8s.io images import /tmp/${IMAGE_NAME}.tar && rm /tmp/${IMAGE_NAME}.tar"

rm "/tmp/${IMAGE_NAME}.tar"
echo "==> Done! Image ${IMAGE_NAME}:${IMAGE_TAG} is available on ${WORKER}."
