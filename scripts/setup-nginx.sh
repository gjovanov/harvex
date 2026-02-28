#!/bin/bash
set -euo pipefail

# Install nginx reverse proxy config on mars for harvex.lgrai.app
# Run this script ON mars with sudo

DOMAIN="harvex.lgrai.app"
CONF="/etc/nginx/sites-available/${DOMAIN}.conf"
# worker2 WireGuard IP
UPSTREAM="10.10.20.11:30060"

echo "==> Writing nginx config to ${CONF}..."
cat > "${CONF}" <<NGINX
server {
    listen 80;
    server_name ${DOMAIN};
    return 301 https://\$host\$request_uri;
}

server {
    listen 443 ssl http2;
    server_name ${DOMAIN};

    ssl_certificate /etc/nginx/ssl/${DOMAIN}.crt;
    ssl_certificate_key /etc/nginx/ssl/${DOMAIN}.key;

    client_max_body_size 100m;

    location / {
        proxy_pass http://${UPSTREAM};
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_read_timeout 600;
        proxy_send_timeout 600;
    }

    # SSE progress stream
    location ~ ^/api/batch/.*/progress\$ {
        proxy_pass http://${UPSTREAM};
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 86400;
        chunked_transfer_encoding off;
    }
}
NGINX

echo "==> Enabling site..."
ln -sf "${CONF}" "/etc/nginx/sites-enabled/${DOMAIN}.conf"

echo "==> Testing nginx config..."
nginx -t

echo "==> Reloading nginx..."
systemctl reload nginx

echo "==> Done! ${DOMAIN} -> ${UPSTREAM}"
