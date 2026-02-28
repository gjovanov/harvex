#!/bin/bash
set -euo pipefail

# Deploy Harvex to Kubernetes
cd "$(dirname "$0")/.."

echo "==> Applying namespace..."
kubectl apply -f k8s/namespace.yml

echo "==> Applying deployment, configmap, and service..."
kubectl apply -f k8s/deployment.yml

echo "==> Waiting for rollout..."
kubectl -n harvex rollout status deployment/harvex --timeout=120s

echo "==> Deployment status:"
kubectl -n harvex get pods
kubectl -n harvex get svc

echo "==> Done! Harvex is running on NodePort 30060."
