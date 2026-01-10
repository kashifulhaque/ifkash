#!/bin/bash
set -e

echo "🚀 Setting up ifkash development environment..."

# Ensure mise is available
if ! command -v mise &> /dev/null; then
    echo "📦 Installing mise..."
    curl https://mise.run | sh
    export PATH="$HOME/.local/bin:$PATH"
fi

# Ensure mise cache directories exist (should already be created by Dockerfile)
mkdir -p /workspace/.mise/cache /workspace/.mise/installs /workspace/.mise/downloads

# Trust the mise.toml configuration
echo "✅ Trusting mise configuration..."
mise trust

# Note: All tools (Rust, Bun, Node.js, Typst, Wrangler) are pre-installed in the Dockerfile
# Verify tools are available
echo "🔍 Verifying pre-installed tools..."
echo "  - Rust: $(rustc --version 2>/dev/null || echo 'not found')"
echo "  - Cargo: $(cargo --version 2>/dev/null || echo 'not found')"
echo "  - Bun: $(bun --version 2>/dev/null || echo 'not found')"
echo "  - Node: $(node --version 2>/dev/null || echo 'not found')"
echo "  - Typst: $(typst --version 2>/dev/null || echo 'not found')"
echo "  - Wrangler: $(wrangler --version 2>/dev/null || echo 'not found')"

# Activate mise in current shell
eval "$(mise activate bash)"

# Install frontend dependencies
echo "📦 Installing frontend dependencies..."
bun install

# Install API dependencies (Rust)
echo "📦 Setting up Rust API..."
cd api
rustup target add wasm32-unknown-unknown || true
cargo install -q worker-build || echo "worker-build already installed"
cd ..

# Download fonts for Typst
echo "🔤 Downloading fonts..."
mise run assets:fonts

# Create necessary directories
echo "📁 Creating necessary directories..."
mkdir -p .wrangler/state
mkdir -p static/assets

# Set up git hooks (if any)
if [ -d ".git" ]; then
    echo "🔧 Setting up git configuration..."
    git config --local core.autocrlf input
fi

echo "✅ Setup complete! You can now run:"
echo "   - Frontend: mise run web:dev (or bun run dev)"
echo "   - Backend API: mise run api:dev"
echo "   - Typst Service: cd typst-service && bun run dev"
echo ""
echo "Or use the start-services.sh script to run all services."
