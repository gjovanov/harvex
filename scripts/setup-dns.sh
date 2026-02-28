#!/bin/bash
set -euo pipefail

# Create Cloudflare DNS A record for harvex.lgrai.app
# Uses the lgrai.app zone (same as LGR project)

DOMAIN="harvex.lgrai.app"
# mars public IP
IP="94.130.141.98"
# lgrai.app Cloudflare zone ID (from LGR deploy)
ZONE_ID="${CF_ZONE_ID:?Set CF_ZONE_ID for lgrai.app}"
CF_TOKEN="${CF_API_TOKEN:?Set CF_API_TOKEN}"

echo "==> Creating DNS A record: ${DOMAIN} -> ${IP}"
curl -s -X POST "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/dns_records" \
  -H "Authorization: Bearer ${CF_TOKEN}" \
  -H "Content-Type: application/json" \
  --data "{
    \"type\": \"A\",
    \"name\": \"harvex\",
    \"content\": \"${IP}\",
    \"ttl\": 1,
    \"proxied\": true
  }" | python3 -m json.tool

echo "==> Done! ${DOMAIN} -> ${IP} (proxied via Cloudflare)"
