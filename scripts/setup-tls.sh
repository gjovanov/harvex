#!/bin/bash
set -euo pipefail

# TLS for harvex.lgrai.app is handled by the *.lgrai.app wildcard cert
# managed by the lgr-deploy project. No separate cert is needed.
#
# If the wildcard cert needs renewal, run:
#   cd /home/gjovanov/lgr-deploy && sudo ./scripts/setup-certs.sh
#
# Verify coverage:
#   openssl x509 -in /gjovanov/nginx/cert/lgrai.app.pem -text -noout | grep DNS

echo "==> harvex.lgrai.app uses the *.lgrai.app wildcard cert"
echo "==> Checking cert coverage..."
openssl x509 -in /gjovanov/nginx/cert/lgrai.app.pem -text -noout 2>/dev/null | grep -A1 "Subject Alternative Name"
echo "==> No action needed."
