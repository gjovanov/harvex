#!/bin/bash
set -euo pipefail

# Create Cloudflare DNS A record for harvex.lgrai.app
# Uses the lgrai.app zone (same as LGR project)
cd "$(dirname "$0")/.."

# Load .env if present
if [[ -f .env ]]; then
  set -a; source .env; set +a
fi

DOMAIN="harvex.lgrai.app"
# mars public IP
IP="94.130.141.98"
CF_TOKEN="${CF_Token:?Set CF_Token in .env}"

API_BASE="https://api.cloudflare.com/client/v4"

echo "==> Looking up zone ID for lgrai.app..."
ZONE_ID=$(curl -s "${API_BASE}/zones?name=lgrai.app&status=active" \
  -H "Authorization: Bearer ${CF_TOKEN}" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['result'][0]['id'])")
echo "    Zone ID: ${ZONE_ID}"

echo "==> Creating DNS A record: ${DOMAIN} -> ${IP}"
curl -s -X POST "${API_BASE}/zones/${ZONE_ID}/dns_records" \
  -H "Authorization: Bearer ${CF_TOKEN}" \
  -H "Content-Type: application/json" \
  --data "{
    \"type\": \"A\",
    \"name\": \"harvex\",
    \"content\": \"${IP}\",
    \"ttl\": 1,
    \"proxied\": false
  }" | python3 -m json.tool

echo "==> Done! ${DOMAIN} -> ${IP}"
