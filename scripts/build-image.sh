#!/bin/bash
set -euo pipefail

# Build Docker image, export, transfer to worker2, and import into containerd
cd "$(dirname "$0")/.."

# Load .env if present
if [[ -f .env ]]; then
  set -a; source .env; set +a
fi

IMAGE_NAME="harvex"
IMAGE_TAG="latest"
TAR_FILE="/tmp/${IMAGE_NAME}.tar"

K8S_SSH_KEY="${K8S_SSH_KEY:?Set K8S_SSH_KEY in .env}"
K8S_WORKER2_IP="${K8S_WORKER2_IP:?Set K8S_WORKER2_IP in .env}"

# SSH options (proxy through zeus for worker-2)
SSH_OPTS=(-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null)
if [[ -n "${K8S_SSH_PROXY_CMD:-}" ]]; then
  SSH_OPTS+=(-o "ProxyCommand=$K8S_SSH_PROXY_CMD")
fi

do_ssh() { ssh "${SSH_OPTS[@]}" -i "$K8S_SSH_KEY" "ubuntu@$K8S_WORKER2_IP" "$@"; }
do_scp() { scp "${SSH_OPTS[@]}" -i "$K8S_SSH_KEY" "$@"; }

echo "==> Building Docker image ${IMAGE_NAME}:${IMAGE_TAG}..."
docker build -t "${IMAGE_NAME}:${IMAGE_TAG}" .

echo "==> Exporting image to tarball..."
docker save "${IMAGE_NAME}:${IMAGE_TAG}" -o "$TAR_FILE"

echo "==> Transferring to worker2 (${K8S_WORKER2_IP})..."
do_scp "$TAR_FILE" "ubuntu@${K8S_WORKER2_IP}:/tmp/"

echo "==> Importing into containerd on worker2..."
do_ssh "sudo ctr -n k8s.io images import /tmp/${IMAGE_NAME}.tar && rm /tmp/${IMAGE_NAME}.tar"

rm "$TAR_FILE"
echo "==> Done! Image ${IMAGE_NAME}:${IMAGE_TAG} is available on worker2."
