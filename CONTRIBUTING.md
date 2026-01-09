# Contributing to ifkash

Thank you for your interest in contributing! This guide will help you get started with the development environment.

## 🚀 Quick Start

### Using DevContainer (Recommended)

The project includes a fully configured development container that sets up all dependencies automatically.

#### Prerequisites
- [Docker Desktop](https://www.docker.com/products/docker-desktop)
- [VS Code](https://code.visualstudio.com/)
- [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)

#### Steps
1. Clone the repository
2. Open the project in VS Code
3. When prompted, click "Reopen in Container"
4. Wait for the container to build and setup to complete
5. All services will start automatically!

### Using GitHub Codespaces

1. Click "Code" → "Create codespace on main"
2. Wait for the environment to set up
3. Start coding!

## 🏗️ Project Structure

```
ifkash/
├── api/                    # Rust-based Cloudflare Worker (Backend)
├── src/                    # SvelteKit application (Frontend)
├── typst-service/          # Typst compilation service
├── .devcontainer/          # DevContainer configuration
│   ├── devcontainer.json   # Main devcontainer config
│   ├── Dockerfile          # Container image definition
│   ├── setup.sh            # Post-create setup script
│   ├── start-services.sh   # Start all services
│   ├── stop-services.sh    # Stop all services
│   └── health-check.sh     # Verify environment health
└── mise.toml               # Tool version management
```

## 🛠️ Development Workflow

### Running Services

All services are automatically started when the devcontainer starts. You can also control them manually:

**Start all services:**
```bash
bash .devcontainer/start-services.sh
```

**Stop all services:**
```bash
bash .devcontainer/stop-services.sh
```

**Start individual services:**
```bash
# Frontend (SvelteKit)
mise run web:dev

# Backend API (Cloudflare Worker)
mise run api:dev

# Typst Service
cd typst-service && bun run dev
```

### Using VS Code Tasks

The project includes pre-configured VS Code tasks. Access them via:
- Command Palette (Cmd/Ctrl + Shift + P) → "Tasks: Run Task"

Available tasks:
- Start All Services
- Stop All Services
- Frontend: Dev
- Backend: Dev
- Typst Service: Dev
- View Frontend Logs
- View Backend Logs
- View Typst Service Logs

### Viewing Logs

Service logs are stored in `.devcontainer/logs/`:

```bash
# View frontend logs
tail -f .devcontainer/logs/frontend.log

# View backend logs
tail -f .devcontainer/logs/backend-api.log

# View typst service logs
tail -f .devcontainer/logs/typst-service.log
```

## 🧪 Testing

### Frontend
```bash
bun run check
```

### Backend (Rust)
```bash
cd api
cargo test
```

## 📦 Building for Production

### Frontend
```bash
bun run build
```

### Backend
```bash
mise run api:build
```

## 🔍 Health Check

Run the health check script to verify your environment:

```bash
bash .devcontainer/health-check.sh
```

This will check:
- ✅ Required tools are installed
- ✅ Rust targets are configured
- ✅ Dependencies are installed
- ✅ Services are running
- ✅ Fonts are downloaded

## 📝 Code Style

### TypeScript/Svelte
- Use Prettier for formatting (configured to run on save)
- Follow the existing code style

### Rust
- Use `rustfmt` for formatting
- Run `cargo clippy` before committing

### Typst
- Use Typst LSP formatter

## 🐛 Debugging

### Frontend (SvelteKit)
Use browser DevTools or VS Code's built-in debugger.

### Backend (Rust Worker)
Use the LLDB debugger configuration in `.vscode/launch.json`:
1. Set breakpoints in your Rust code
2. Press F5 or use "Debug Rust Worker" launch configuration

### Typst Service
Use the Node debugger configuration in `.vscode/launch.json`:
1. Set breakpoints in `typst-service/index.ts`
2. Use "Debug Typst Service" launch configuration

## 🔧 Troubleshooting

### Container won't start
1. Ensure Docker Desktop is running
2. Try rebuilding: Command Palette → "Dev Containers: Rebuild Container"

### Services won't start
1. Check if ports are already in use: `lsof -i :5173 -i :8787 -i :3001`
2. Run health check: `bash .devcontainer/health-check.sh`
3. Check logs in `.devcontainer/logs/`

### Missing dependencies
```bash
# Reinstall frontend dependencies
bun install

# Reinstall Rust dependencies
cd api && cargo build
```

### Fonts not working
```bash
mise run assets:fonts
```

## 📚 Additional Resources

- [mise Documentation](https://mise.jdx.dev/)
- [SvelteKit Documentation](https://kit.svelte.dev/)
- [Cloudflare Workers Documentation](https://developers.cloudflare.com/workers/)
- [Typst Documentation](https://typst.app/docs/)
- [Dev Containers Documentation](https://code.visualstudio.com/docs/devcontainers/containers)

## 🤝 Submitting Changes

1. Create a new branch for your feature/fix
2. Make your changes
3. Test thoroughly
4. Run health check to ensure everything works
5. Commit with clear, descriptive messages
6. Push and create a Pull Request

## 💡 Tips

- **Performance**: The devcontainer uses volume mounts for caching to improve performance
- **Port Forwarding**: Ports are automatically forwarded in VS Code
- **Environment Variables**: Configure in `.devcontainer/devcontainer.json` under `remoteEnv`
- **Codespaces**: The configuration is optimized for both local and Codespaces development

## 📧 Questions?

If you have questions or run into issues, please open an issue on GitHub.

Happy coding! 🎉
