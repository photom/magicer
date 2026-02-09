# Deployment Guide - Linux File Magic API Server <!-- omit in toc -->

- [1. System Requirements](#1-system-requirements)
- [2. Build Configuration](#2-build-configuration)
- [3. Runtime Configuration](#3-runtime-configuration)
- [4. Environment Variables](#4-environment-variables)
- [5. Docker Deployment](#5-docker-deployment)
- [6. Systemd Service](#6-systemd-service)
- [7. Monitoring and Health Checks](#7-monitoring-and-health-checks)
- [8. Security Hardening](#8-security-hardening)
- [9. Performance Tuning](#9-performance-tuning)
- [10. Logging Configuration](#10-logging-configuration)

---

## 1. System Requirements

### Operating System
- **Platform:** Linux x86_64 (kernel 5.10+)
- **Distributions:** Ubuntu 22.04+, Debian 12+, RHEL 9+, Alpine 3.18+

### Dependencies
- `libmagic1` (runtime library)
- `libmagic-dev` (build-time headers)
- `file` package (provides magic database files)

**Installation:**
```bash
# Debian/Ubuntu
sudo apt-get update
sudo apt-get install -y libmagic1 libmagic-dev file

# RHEL/CentOS/Rocky
sudo dnf install -y file-libs file-devel file

# Alpine
apk add --no-cache libmagic file
```

### Resource Requirements

| Environment | CPU | Memory | Disk |
|------------|-----|--------|------|
| Development | 2 cores | 512MB | 100MB |
| Production (1k conn) | 4 cores | 2GB | 500MB |
| Production (high load) | 8 cores | 4GB | 1GB |

---

## 2. Build Configuration

### Development Build
```bash
cargo build --bin magicer
```

### Production Build
```bash
# Optimized release build
RUSTFLAGS="-C target-cpu=native -C link-arg=-s" \
  cargo build --release --bin magicer

# Strip debug symbols (if not using -s flag)
strip target/release/magicer
```

### Cross-Compilation
```bash
# For Alpine Linux (musl target)
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

### Build Verification
```bash
# Check binary dependencies
ldd target/release/magicer

# Expected output should include:
# libmagic.so.1 => /usr/lib/x86_64-linux-gnu/libmagic.so.1
# libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6
```

---

## 3. Runtime Configuration

### Configuration File

Create `/etc/magicer/config.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8080
max_connections = 1000
backlog = 1024

[server.timeouts]
read_timeout_secs = 60
write_timeout_secs = 60
analysis_timeout_secs = 30
keepalive_secs = 75

[server.limits]
max_body_size_mb = 100
max_uri_length = 8192
max_header_size = 16384

[sandbox]
# Directory for path-based analysis (relative path root)
base_dir = "/var/lib/magicer/files"

[magic]
# Optional: custom magic database path
# If not set, uses system default (/usr/share/misc/magic.mgc)
database_path = "/usr/share/misc/magic.mgc"

[logging]
level = "info"
format = "json"  # Options: "json", "pretty", "compact"
```

### Directory Structure

```bash
/etc/magicer/
  ├── config.toml          # Main configuration
  └── credentials.env      # Secrets (0600 permissions)

/var/lib/magicer/
  └── files/               # Sandbox base directory (path analysis)

/var/log/magicer/
  └── server.log           # Application logs
```

**Setup:**
```bash
sudo mkdir -p /etc/magicer /var/lib/magicer/files /var/log/magicer
sudo chown -R magicer:magicer /var/lib/magicer /var/log/magicer
sudo chmod 755 /var/lib/magicer/files
```

---

## 4. Environment Variables

### Required Variables

```bash
# Authentication credentials (production)
export MAGICER_AUTH_USERNAME="api_user"
export MAGICER_AUTH_PASSWORD="secure_password_here"

# Configuration file path
export MAGICER_CONFIG_PATH="/etc/magicer/config.toml"
```

### Optional Variables

```bash
# Override listen address
export MAGICER_HOST="127.0.0.1"
export MAGICER_PORT="8080"

# Sandbox directory
export MAGICER_SANDBOX_DIR="/var/lib/magicer/files"

# Logging
export RUST_LOG="magicer=info,tower_http=debug"
export MAGICER_LOG_FORMAT="json"

# Performance tuning
export TOKIO_WORKER_THREADS="4"
```

### Credentials Management

**Production Setup (using systemd EnvironmentFile):**
```bash
# /etc/magicer/credentials.env (mode 0600)
MAGICER_AUTH_USERNAME=api_user
MAGICER_AUTH_PASSWORD=$(openssl rand -base64 32)
```

**Verify Permissions:**
```bash
sudo chmod 600 /etc/magicer/credentials.env
sudo chown root:root /etc/magicer/credentials.env
```

---

## 5. Docker Deployment

### Dockerfile

**Multi-stage build:**
```dockerfile
# Build stage
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    libmagic-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Build application
COPY . .
RUN touch src/main.rs  # Force rebuild
RUN cargo build --release --bin magicer

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libmagic1 \
    file \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash magicer

# Setup directories
RUN mkdir -p /etc/magicer /var/lib/magicer/files /var/log/magicer && \
    chown -R magicer:magicer /var/lib/magicer /var/log/magicer

# Copy binary
COPY --from=builder /build/target/release/magicer /usr/local/bin/magicer
RUN chmod +x /usr/local/bin/magicer

# Switch to non-root user
USER magicer
WORKDIR /home/magicer

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["/usr/local/bin/magicer", "health-check"] || exit 1

# Run server
CMD ["/usr/local/bin/magicer", "serve"]
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  magicer:
    build:
      context: .
      dockerfile: Dockerfile
    image: magicer:latest
    container_name: magicer
    restart: unless-stopped
    
    ports:
      - "8080:8080"
    
    environment:
      - MAGICER_HOST=0.0.0.0
      - MAGICER_PORT=8080
      - RUST_LOG=magicer=info
      - MAGICER_LOG_FORMAT=json
    
    env_file:
      - .env.production
    
    volumes:
      - ./config/config.toml:/etc/magicer/config.toml:ro
      - magicer-files:/var/lib/magicer/files
      - magicer-logs:/var/log/magicer
    
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/v1/ping"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
    
    security_opt:
      - no-new-privileges:true
    
    read_only: true
    tmpfs:
      - /tmp:noexec,nosuid,size=100m
    
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE

volumes:
  magicer-files:
  magicer-logs:
```

### Running with Docker

```bash
# Build image
docker build -t magicer:latest .

# Run container
docker run -d \
  --name magicer \
  -p 8080:8080 \
  -e MAGICER_AUTH_USERNAME=admin \
  -e MAGICER_AUTH_PASSWORD=secret \
  -v $(pwd)/config.toml:/etc/magicer/config.toml:ro \
  magicer:latest

# View logs
docker logs -f magicer

# Health check
curl http://localhost:8080/v1/ping
```

---

## 6. Systemd Service

### Service Unit File

Create `/etc/systemd/system/magicer.service`:

```ini
[Unit]
Description=Linux File Magic API Server
Documentation=https://github.com/yourusername/magicer
After=network.target
Wants=network-online.target

[Service]
Type=notify
User=magicer
Group=magicer

# Binary location
ExecStart=/usr/local/bin/magicer serve

# Configuration
Environment="MAGICER_CONFIG_PATH=/etc/magicer/config.toml"
EnvironmentFile=/etc/magicer/credentials.env

# Working directory
WorkingDirectory=/var/lib/magicer

# Restart policy
Restart=on-failure
RestartSec=5s
KillMode=mixed
KillSignal=SIGTERM
TimeoutStopSec=30s

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/magicer /var/log/magicer
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictNamespaces=true
SystemCallFilter=@system-service
SystemCallErrorNumber=EPERM

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=magicer

[Install]
WantedBy=multi-user.target
```

### Service Management

```bash
# Install service
sudo cp magicer.service /etc/systemd/system/
sudo systemctl daemon-reload

# Enable auto-start
sudo systemctl enable magicer

# Start service
sudo systemctl start magicer

# Check status
sudo systemctl status magicer

# View logs
sudo journalctl -u magicer -f

# Restart after config change
sudo systemctl reload-or-restart magicer

# Stop service
sudo systemctl stop magicer
```

---

## 7. Monitoring and Health Checks

### Health Check Endpoint

```bash
# Basic health check
curl -f http://localhost:8080/v1/ping

# Expected response (200 OK):
{
  "message": "pong",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Monitoring Script

```bash
#!/bin/bash
# /usr/local/bin/magicer-healthcheck.sh

ENDPOINT="http://localhost:8080/v1/ping"
TIMEOUT=5

if curl -f -s -m $TIMEOUT "$ENDPOINT" > /dev/null; then
    echo "OK"
    exit 0
else
    echo "FAILED"
    exit 1
fi
```

### Prometheus Metrics (Future)

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'magicer'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/v1/metrics'
    scrape_interval: 15s
```

---

## 8. Security Hardening

### Firewall Configuration

```bash
# UFW (Ubuntu)
sudo ufw allow 8080/tcp comment 'Magicer API'
sudo ufw enable

# iptables
sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT
sudo iptables-save > /etc/iptables/rules.v4
```

### TLS Termination (Reverse Proxy)

**Nginx Configuration:**
```nginx
upstream magicer_backend {
    server 127.0.0.1:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 443 ssl http2;
    server_name api.example.com;

    ssl_certificate /etc/ssl/certs/api.example.com.crt;
    ssl_certificate_key /etc/ssl/private/api.example.com.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    client_max_body_size 100M;
    proxy_read_timeout 60s;
    proxy_send_timeout 60s;

    location /v1/ {
        proxy_pass http://magicer_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### SELinux Policy (RHEL/CentOS)

```bash
# Allow network binding
sudo setsebool -P httpd_can_network_connect 1

# Create custom policy if needed
sudo semanage port -a -t http_port_t -p tcp 8080
```

---

## 9. Performance Tuning

### Kernel Parameters

```bash
# /etc/sysctl.d/99-magicer.conf

# Increase TCP backlog
net.core.somaxconn = 2048
net.ipv4.tcp_max_syn_backlog = 2048

# TCP tuning
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_keepalive_time = 75
net.ipv4.tcp_keepalive_intvl = 15
net.ipv4.tcp_keepalive_probes = 5

# File descriptors
fs.file-max = 100000
```

Apply:
```bash
sudo sysctl -p /etc/sysctl.d/99-magicer.conf
```

### Process Limits

```bash
# /etc/security/limits.d/magicer.conf
magicer soft nofile 65536
magicer hard nofile 65536
magicer soft nproc 4096
magicer hard nproc 4096
```

---

## 10. Logging Configuration

### JSON Structured Logs

```bash
# Environment variable
export MAGICER_LOG_FORMAT=json

# Example log entry:
{
  "timestamp": "2026-02-09T12:34:56.789Z",
  "level": "INFO",
  "target": "magicer::presentation::handlers",
  "message": "Request completed",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": 200,
  "duration_ms": 45,
  "path": "/v1/magic/content"
}
```

### Log Rotation (logrotate)

```bash
# /etc/logrotate.d/magicer
/var/log/magicer/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 0644 magicer magicer
    sharedscripts
    postrotate
        systemctl reload magicer > /dev/null 2>&1 || true
    endscript
}
```

### Centralized Logging

**Fluentd/Fluent Bit Configuration:**
```conf
[INPUT]
    Name              tail
    Path              /var/log/magicer/server.log
    Parser            json
    Tag               magicer.server
    Refresh_Interval  5

[FILTER]
    Name    record_modifier
    Match   magicer.*
    Record  service magicer
    Record  environment production

[OUTPUT]
    Name  es
    Match magicer.*
    Host  elasticsearch.internal
    Port  9200
    Index magicer-logs
```

---

## Summary

This deployment guide provides production-ready configurations for:
- System requirements and build optimization
- Docker containerization with security hardening
- Systemd service management
- Monitoring and health checks
- Security best practices
- Performance tuning recommendations

For questions or issues, refer to the project repository or open an issue.
