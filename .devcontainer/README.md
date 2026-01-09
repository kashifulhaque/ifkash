# ifkash DevContainer

This devcontainer provides a complete development environment for the ifkash project, including:

- **Frontend**: SvelteKit with Bun
- **Backend API**: Rust-based Cloudflare Worker
- **Typst Service**: Bun-based Typst compilation service
- **Tools**: mise for managing Node.js, Bun, Rust, Typst, and Wrangler

## 🚀 Quick Start

### Local Development

1. Open the project in VS Code
2. When prompted, click "Reopen in Container"
3. Wait for the container to build and setup to complete
4. Services will start automatically!

### GitHub Codespaces

1. Create a new Codespace from the repository
2. Wait for the environment to set up
3. Services will start automatically!

## 📊 Services

All services are automatically started when the container starts:

| Service | Port | URL |
|---------|------|-----|
| Frontend (SvelteKit) | 5173 | http://localhost:5173 |
| Backend API (Cloudflare Worker) | 8787 | http://localhost:8787 |
| Typst Service | 3001 | http://localhost:3001 |

## 🛠️ Manual Service Control

### Start All Services
```bash
bash .devcontainer/start-services.sh
```

### Stop All Services
```bash
bash .devcontainer/stop-services.sh
```

### Start Individual Services

**Frontend:**
```bash
mise run web:dev
# or
bun run dev
```

**Backend API:**
```bash
mise run api:dev
# or
cd api && npx wrangler dev --persist-to .wrangler/state
```

**Typst Service:**
```bash
cd typst-service && bun run dev
```

## 📝 Available mise Tasks

View all available tasks:
```bash
mise tasks
```

Common tasks:
- `mise run web:dev` - Start SvelteKit frontend
- `mise run api:dev` - Start Cloudflare Worker API
- `mise run api:build` - Build the Worker for production
- `mise run assets:fonts` - Download required fonts
- `mise run assets:typst` - Compile Typst files to PDF

## 🔧 Installed Tools

The devcontainer comes pre-configured with:

- **Bun** (latest) - JavaScript runtime and package manager
- **Node.js** (LTS) - JavaScript runtime
- **Rust** (stable) - Systems programming language
- **Typst** (latest) - Markup-based typesetting system
- **Wrangler** (latest) - Cloudflare Workers CLI
- **mise** - Tool version manager

## 📦 VS Code Extensions

Pre-installed extensions:
- Svelte for VS Code
- Tailwind CSS IntelliSense
- rust-analyzer
- Typst LSP
- GitLens
- GitHub Copilot (if available)

## 🐛 Debugging

### View Service Logs

Logs are stored in `.devcontainer/logs/`:
```bash
# Frontend logs
tail -f .devcontainer/logs/frontend.log

# Backend API logs
tail -f .devcontainer/logs/backend-api.log

# Typst Service logs
tail -f .devcontainer/logs/typst-service.log
```

### Check Service Status

```bash
# Check if services are running
ps aux | grep -E "(vite|wrangler|bun)"

# Check port usage
lsof -i :5173  # Frontend
lsof -i :8787  # Backend
lsof -i :3001  # Typst
```

## 🔄 Rebuilding the Container

If you need to rebuild the container:

1. Open Command Palette (Cmd/Ctrl + Shift + P)
2. Select "Dev Containers: Rebuild Container"
3. Wait for the rebuild to complete

## 📚 Additional Resources

- [mise Documentation](https://mise.jdx.dev/)
- [SvelteKit Documentation](https://kit.svelte.dev/)
- [Cloudflare Workers Documentation](https://developers.cloudflare.com/workers/)
- [Typst Documentation](https://typst.app/docs/)

## 🤝 Contributing

When developing in the devcontainer:

1. All changes are synced to your local filesystem
2. Git operations work normally
3. Use the integrated terminal for all commands
4. Extensions are automatically configured

## 💡 Tips

- **Performance**: The container uses volume mounts for caching (cargo, bun, node_modules) to improve performance
- **Port Forwarding**: Ports are automatically forwarded and labeled in VS Code
- **Environment Variables**: Configure in `.devcontainer/devcontainer.json` under `remoteEnv`
- **GitHub Codespaces**: The configuration is optimized for both local and Codespaces development
