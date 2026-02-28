#!/bin/bash
set -euo pipefail

# Issue TLS certificate for harvex.lgrai.app via acme.sh on mars
# Run this script ON mars (or ssh to mars first)

DOMAIN="harvex.lgrai.app"
CF_TOKEN="${CF_API_TOKEN:?Set CF_API_TOKEN}"

echo "==> Issuing TLS certificate for ${DOMAIN}..."
export CF_Token="${CF_TOKEN}"
~/.acme.sh/acme.sh --issue --dns dns_cf -d "${DOMAIN}"

echo "==> Installing certificate to nginx..."
~/.acme.sh/acme.sh --install-cert -d "${DOMAIN}" \
  --key-file "/etc/nginx/ssl/${DOMAIN}.key" \
  --fullchain-file "/etc/nginx/ssl/${DOMAIN}.crt" \
  --reloadcmd "systemctl reload nginx"

echo "==> Done!"
