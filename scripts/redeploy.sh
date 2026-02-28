#!/bin/bash
set -euo pipefail

# Rebuild image, transfer, and restart deployment
cd "$(dirname "$0")/.."

./scripts/build-image.sh

echo "==> Restarting deployment..."
kubectl -n harvex rollout restart deployment/harvex
kubectl -n harvex rollout status deployment/harvex --timeout=120s

echo "==> Pods:"
kubectl -n harvex get pods

echo "==> Done!"
