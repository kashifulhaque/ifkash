# Typst Compilation Service (Docker)

A lightweight HTTP service for compiling Typst documents to PDF, containerized with Docker and integrated with Caddy reverse proxy.

## Quick Start

### 1. Copy to Your VM

```bash
scp -r typst-service/ your-vm:~/
```

### 2. Deploy

```bash
ssh your-vm
cd ~/typst-service
./deploy-docker.sh
```

The script will:
- ✅ Check Docker installation
- ✅ Download fonts
- ✅ Build Docker image
- ✅ Start the service
- ✅ Update Caddyfile
- ✅ Reload Caddy

### 3. Test

```bash
# Health check
curl https://typst.ifkash.dev/health

# Compile test
curl -X POST https://typst.ifkash.dev/compile \
  -H "Authorization: Bearer test" \
  -H "Content-Type: application/json" \
  -d '{"code": "#set page(width: 10cm, height: 10cm)\nHello World"}' \
  -o test.pdf
```

## Architecture

```
Internet → Caddy (443) → typst-service:3001 → Typst Compiler → PDF
           (typst.ifkash.dev)
```

## Files

- `Dockerfile` - Container image definition
- `compose.yml` - Service configuration
- `index.ts` - Bun HTTP server
- `package.json` - Dependencies
- `deploy-docker.sh` - Automated deployment
- `fonts/` - Font files (mounted as volume)

## Configuration

### Caddyfile Entry

Add this to your Caddyfile (done automatically by deploy script):

```
typst.ifkash.dev {
  reverse_proxy typst-service:3001
}
```

### Resource Limits

Edit `compose.yml`:

```yaml
deploy:
  resources:
    limits:
      cpus: '0.5'      # Max 50% CPU
      memory: 512M     # Max 512MB RAM
```

### Environment Variables

Edit `compose.yml`:

```yaml
environment:
  - PORT=3001
  - FONT_PATH=/app/fonts
```

## Management

### View Logs

```bash
cd ~/typst-service
docker compose logs -f
```

### Restart Service

```bash
docker compose restart
```

### Stop Service

```bash
docker compose down
```

### Rebuild After Changes

```bash
docker compose build
docker compose up -d
```

### Update Typst Version

Edit `Dockerfile` and change the version:

```dockerfile
RUN curl -fsSL https://github.com/typst/typst/releases/download/v0.13.0/...
```

Then rebuild:

```bash
docker compose build
docker compose up -d
```

## Adding Custom Fonts

1. Copy fonts to the `fonts/` directory:
   ```bash
   cp my-font.ttf ~/typst-service/fonts/
   ```

2. Restart the service:
   ```bash
   docker compose restart
   ```

## Monitoring

### Service Status

```bash
docker compose ps
```

### Resource Usage

```bash
docker stats typst-service
```

### Health Check

```bash
curl http://localhost:3001/health
```

## Troubleshooting

### Service won't start

```bash
# Check logs
docker compose logs

# Common issues:
# - Port 3001 already in use
# - web_network doesn't exist
# - Fonts directory not mounted
```

### Compilation fails

```bash
# Test Typst directly in container
docker compose exec typst-service typst --version

# Check fonts
docker compose exec typst-service ls -la /app/fonts
```

### Can't access via Caddy

```bash
# Check if service is on web_network
docker network inspect web_network

# Reload Caddy
cd ~/caddy
docker compose exec caddy caddy reload --config /etc/caddy/Caddyfile
```

## Integration with Existing Services

Your VM setup:

```
┌─────────────────────────────────────────┐
│ Caddy (Port 80/443)                     │
│  ├─ boo.ifkash.dev → manager:8080       │
│  ├─ pdf.ifkash.dev → stirling-pdf:8080  │
│  ├─ pb.ifkash.dev → pocketbase:8090     │
│  └─ typst.ifkash.dev → typst-service:3001│
└─────────────────────────────────────────┘
```

All services share the `web_network` Docker network.

## Security

- ✅ Authentication via Bearer token
- ✅ Rate limiting (10 req/min per IP)
- ✅ Timeout protection (30s max)
- ✅ Resource limits (CPU, memory)
- ✅ HTTPS via Caddy
- ✅ No file system access from Typst
- ✅ Temporary file cleanup

## Performance

- **Compilation time**: < 2s for typical resume
- **Memory usage**: ~50-100MB per compilation
- **Container size**: ~150MB
- **Startup time**: < 5s

## Backup

```bash
# Backup fonts
tar -czf fonts-backup.tar.gz ~/typst-service/fonts

# Backup configuration
tar -czf typst-config-backup.tar.gz ~/typst-service/*.yml ~/typst-service/Dockerfile
```

## License

MIT
