# DevContainer Quick Reference

## 🚀 Getting Started

### Local Development
```bash
# 1. Open in VS Code
# 2. Click "Reopen in Container"
# 3. Wait for setup
# 4. Services auto-start!
```

### GitHub Codespaces
```bash
# 1. Click "Code" → "Create codespace"
# 2. Wait for setup
# 3. Start coding!
```

## 📊 Service URLs

| Service | URL | Port |
|---------|-----|------|
| Frontend | http://localhost:5173 | 5173 |
| Backend API | http://localhost:8787 | 8787 |
| Typst Service | http://localhost:3001 | 3001 |

## 🛠️ Common Commands

### Service Management
```bash
# Start all services
bash .devcontainer/start-services.sh

# Stop all services
bash .devcontainer/stop-services.sh

# Health check
bash .devcontainer/health-check.sh
```

### Individual Services
```bash
# Frontend
mise run web:dev

# Backend
mise run api:dev

# Typst
cd typst-service && bun run dev
```

### View Logs
```bash
# Frontend
tail -f .devcontainer/logs/frontend.log

# Backend
tail -f .devcontainer/logs/backend-api.log

# Typst
tail -f .devcontainer/logs/typst-service.log
```

### Build
```bash
# Frontend
bun run build

# Backend
mise run api:build
```

## 🔧 VS Code Tasks

Access via: `Cmd/Ctrl + Shift + P` → "Tasks: Run Task"

- Start All Services
- Stop All Services
- Frontend: Dev
- Backend: Dev
- Typst Service: Dev
- View Frontend Logs
- View Backend Logs
- View Typst Service Logs

## 🐛 Debugging

### Rust Worker
1. Set breakpoints in Rust code
2. Press F5 or use "Debug Rust Worker"

### Typst Service
1. Set breakpoints in `typst-service/index.ts`
2. Use "Debug Typst Service"

## 🔍 Troubleshooting

### Container won't start
```bash
# Rebuild container
# Cmd/Ctrl + Shift + P → "Dev Containers: Rebuild Container"
```

### Services won't start
```bash
# Check ports
lsof -i :5173 -i :8787 -i :3001

# Run health check
bash .devcontainer/health-check.sh

# Check logs
ls -la .devcontainer/logs/
```

### Missing dependencies
```bash
# Frontend
bun install

# Backend
cd api && cargo build
```

### Fonts not working
```bash
mise run assets:fonts
```

## 📦 Installed Tools

- **Bun** (latest) - `bun --version`
- **Node.js** (LTS) - `node --version`
- **Rust** (stable) - `rustc --version`
- **Typst** (latest) - `typst --version`
- **Wrangler** (latest) - `wrangler --version`
- **mise** (latest) - `mise --version`

## 🎯 mise Tasks

```bash
# View all tasks
mise tasks

# Common tasks
mise run web:dev          # Start frontend
mise run api:dev          # Start backend
mise run api:build        # Build backend
mise run assets:fonts     # Download fonts
mise run assets:typst     # Compile Typst files
```

## 📁 Important Directories

```
.devcontainer/          # DevContainer config
.devcontainer/logs/     # Service logs
.vscode/                # VS Code config
.mise/                  # mise cache
.fonts/                 # Downloaded fonts
.wrangler/              # Wrangler state
```

## 🔄 Rebuilding

```bash
# Full rebuild
# Cmd/Ctrl + Shift + P → "Dev Containers: Rebuild Container"

# Rebuild without cache
# Cmd/Ctrl + Shift + P → "Dev Containers: Rebuild Container Without Cache"
```

## 📚 Documentation

- [DevContainer README](.devcontainer/README.md)
- [Setup Summary](.devcontainer/SETUP_SUMMARY.md)
- [Contributing Guide](../CONTRIBUTING.md)
- [Main README](../README.md)

## 💡 Tips

- Services auto-start on container start
- Logs are in `.devcontainer/logs/`
- Use VS Code tasks for common operations
- Health check verifies everything is working
- Volume mounts persist caches across rebuilds

## 🆘 Need Help?

1. Run health check: `bash .devcontainer/health-check.sh`
2. Check logs: `tail -f .devcontainer/logs/*.log`
3. Rebuild container if needed
4. Open an issue on GitHub

---

**Quick Links:**
- [Full Documentation](.devcontainer/README.md)
- [Architecture Diagram](.devcontainer/SETUP_SUMMARY.md)
- [Contributing Guide](../CONTRIBUTING.md)
