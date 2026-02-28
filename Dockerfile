# --- Stage 1: Rust build ---
FROM rust:1.88-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin harvex

# --- Stage 2: Vue SPA build ---
FROM oven/bun:1 AS ui-builder
WORKDIR /app/ui
COPY ui/package.json ui/bun.lock ./
RUN bun install --frozen-lockfile
COPY ui/ .
RUN bun run build

# --- Stage 3: Runtime (nginx + Rust binary) ---
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates nginx && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/harvex /usr/local/bin/
COPY --from=ui-builder /app/ui/dist /var/www/harvex
COPY config/default.toml /app/config/default.toml
COPY files/nginx-pod.conf /etc/nginx/conf.d/default.conf
COPY files/entrypoint.sh /entrypoint.sh
RUN rm -f /etc/nginx/sites-enabled/default && chmod +x /entrypoint.sh

RUN mkdir -p /app/data/db /app/data/uploads

WORKDIR /app
EXPOSE 80
CMD ["/entrypoint.sh"]
