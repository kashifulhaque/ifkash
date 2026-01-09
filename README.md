# [**Kashiful Haque**](https://ifkash.dev) 🧑🏽
This is my personal website 👋

### **Tech stack** 📚
- [Cloudflare Workers](https://workers.cloudflare.com)
- [Cloudflare Pages](https://pages.cloudflare.com)
- [Mise](https://mise.jdx.dev)
  - [Rust](https://rust-lang.org)
  - [Bun](https://bun.sh)
    - [SvelteKit](https://kit.svelte.dev)

### **To run locally** 🏃🏼‍♂️

#### **Option 1: Using DevContainer (Recommended)** 🐳
The easiest way to get started! All dependencies and services are pre-configured.

**Local Development:**
1. Install [Docker Desktop](https://www.docker.com/products/docker-desktop)
2. Install [VS Code](https://code.visualstudio.com/) with [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
3. Open the project in VS Code
4. Click "Reopen in Container" when prompted
5. Wait for setup to complete - all services will start automatically! 🎉

**GitHub Codespaces:**
1. Click "Code" → "Create codespace on main"
2. Wait for the environment to set up
3. All services start automatically! 🚀

**Service URLs:**
- Frontend: http://localhost:5173
- Backend API: http://localhost:8787
- Typst Service: http://localhost:3001

See [.devcontainer/README.md](.devcontainer/README.md) for detailed documentation.

#### **Option 2: Manual Setup** 🛠️
```sh
# Install mise (if not already installed)
curl https://mise.run | sh

# Activate tools and prep once
mise run api:prep

# Start the worker
mise run api:dev

# To run SvelteKit app
mise run web:dev

# To run Typst service
cd typst-service && bun run dev
```

