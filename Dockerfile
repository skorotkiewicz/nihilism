# Build Rust server
FROM rust:alpine AS rust
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN cargo build --release

# Build frontend
FROM oven/bun:alpine AS bun
WORKDIR /build
COPY client/ .
RUN bun install --frozen-lockfile
RUN bun run build

# Runtime
FROM alpine:3.19
RUN apk add --no-cache nginx ca-certificates
WORKDIR /app

COPY --from=rust /build/target/release/nihilism .
COPY --from=bun /build/dist /usr/share/nginx/html

# Create storage directory
RUN mkdir -p /app/data

# Create nginx config
RUN printf '%s\n' \
    'server {' \
    '    listen 8080;' \
    '    root /usr/share/nginx/html;' \
    '    index index.html;' \
    '    ' \
    '    proxy_connect_timeout 1d;' \
    '    proxy_send_timeout 1d;' \
    '    proxy_read_timeout 1d;' \
    '    send_timeout 1d;' \
    '    ' \
    '    location / {' \
    '        try_files $uri $uri/ /index.html;' \
    '    }' \
    '    location /api/ {' \
    '        proxy_pass http://127.0.0.1:3001;' \
    '    }' \
    '}' > /etc/nginx/http.d/default.conf

# Environment variables
ENV HOST=0.0.0.0
ENV PORT=3001
ENV LLM_BASE_URL=http://localhost:8080/v1
ENV LLM_API_KEY=sk-none
ENV LLM_MODEL=gpt-4

# Create start script
RUN printf '%s\n' \
    '#!/bin/sh' \
    'echo "Starting Nihilism server..."' \
    './nihilism &' \
    'echo "✓ API server started"' \
    'echo "✓ Starting nginx"' \
    'nginx -g "daemon off;"' > /app/start.sh && chmod +x /app/start.sh

EXPOSE 8080

CMD ["/app/start.sh"]
