#!/bin/bash
set -euo pipefail

# Install nginx reverse proxy config on mars for harvex.lgrai.app
# Uses the Docker nginx container and existing *.lgrai.app wildcard cert
# Run this script ON mars

DOMAIN="harvex.lgrai.app"
NGINX_CONF_DIR="/gjovanov/nginx/conf.d"
UPSTREAM="10.10.20.11:30060"

echo "==> Writing nginx config to ${NGINX_CONF_DIR}/${DOMAIN}.conf..."
sudo tee "${NGINX_CONF_DIR}/${DOMAIN}.conf" > /dev/null <<NGINX
server {
    listen         80;
    listen         [::]:80;
    server_name    ${DOMAIN};
    return         301 https://${DOMAIN}\$request_uri;
}

server {
    listen 443 ssl http2;
    server_name  ${DOMAIN};

    client_max_body_size 100m;

    gzip on;
    gzip_http_version 1.1;
    gzip_vary on;
    gzip_comp_level 6;
    gzip_proxied any;
    gzip_types text/plain text/css application/json application/javascript application/x-javascript text/javascript;

    brotli_static on;
    brotli on;
    brotli_types text/plain text/css application/json application/javascript application/x-javascript text/javascript;
    brotli_comp_level 4;

    ssl_protocols TLSv1.2 TLSv1.3;

    ssl_certificate /etc/nginx/cert/lgrai.app.pem;
    ssl_certificate_key /etc/nginx/cert/lgrai.app.key;

    ssl_session_cache    shared:SSL:1m;
    ssl_session_timeout  5m;

    ssl_ciphers  HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers  on;

    add_header x-frame-options "deny";
    add_header Strict-Transport-Security "max-age=31536000" always;

    location / {
        proxy_set_header   X-Real-IP \$remote_addr;
        proxy_set_header   Host      \$http_host;
        proxy_pass         http://${UPSTREAM};
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;

        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";

        proxy_read_timeout 300;
        proxy_connect_timeout 300;
        proxy_send_timeout 300;
        send_timeout 300;
    }

    # SSE progress stream — disable buffering
    location ~ ^/api/batch/.*/progress\$ {
        proxy_pass         http://${UPSTREAM};
        proxy_set_header   X-Real-IP \$remote_addr;
        proxy_set_header   Host      \$http_host;
        proxy_set_header   X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header   X-Forwarded-Proto \$scheme;
        proxy_http_version 1.1;
        proxy_set_header   Connection "";
        proxy_buffering    off;
        proxy_cache        off;
        proxy_read_timeout 86400;
        chunked_transfer_encoding off;
    }
}
NGINX

echo "==> Testing nginx config..."
docker exec nginx nginx -t

echo "==> Reloading nginx..."
docker exec nginx nginx -s reload

echo "==> Done! ${DOMAIN} -> ${UPSTREAM}"
