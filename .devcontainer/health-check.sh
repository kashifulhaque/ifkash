#!/bin/bash
set -e

echo "🏥 Health Check for ifkash Development Environment"
echo "=================================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if a command exists
check_command() {
    local cmd=$1
    local name=$2
    
    if command -v $cmd &> /dev/null; then
        echo -e "${GREEN}✓${NC} $name is installed ($(command -v $cmd))"
        return 0
    else
        echo -e "${RED}✗${NC} $name is NOT installed"
        return 1
    fi
}

# Function to check if a port is in use
check_port() {
    local port=$1
    local service=$2
    
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} $service is running on port $port"
        return 0
    else
        echo -e "${YELLOW}⚠${NC} $service is NOT running on port $port"
        return 1
    fi
}

# Check required tools
echo "📦 Checking Required Tools..."
echo "------------------------------"
check_command "mise" "mise"
check_command "bun" "Bun"
check_command "node" "Node.js"
check_command "rustc" "Rust"
check_command "cargo" "Cargo"
check_command "typst" "Typst"
check_command "wrangler" "Wrangler"
echo ""

# Check Rust targets
echo "🦀 Checking Rust Targets..."
echo "------------------------------"
if rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo -e "${GREEN}✓${NC} wasm32-unknown-unknown target is installed"
else
    echo -e "${RED}✗${NC} wasm32-unknown-unknown target is NOT installed"
    echo "   Run: rustup target add wasm32-unknown-unknown"
fi
echo ""

# Check if worker-build is installed
echo "🔧 Checking Cargo Tools..."
echo "------------------------------"
if cargo install --list | grep -q "worker-build"; then
    echo -e "${GREEN}✓${NC} worker-build is installed"
else
    echo -e "${RED}✗${NC} worker-build is NOT installed"
    echo "   Run: cargo install worker-build"
fi
echo ""

# Check if dependencies are installed
echo "📚 Checking Dependencies..."
echo "------------------------------"
if [ -d "node_modules" ]; then
    echo -e "${GREEN}✓${NC} Frontend dependencies are installed"
else
    echo -e "${YELLOW}⚠${NC} Frontend dependencies are NOT installed"
    echo "   Run: bun install"
fi

if [ -d "api/target" ]; then
    echo -e "${GREEN}✓${NC} API has been built at least once"
else
    echo -e "${YELLOW}⚠${NC} API has NOT been built yet"
    echo "   Run: cd api && cargo build"
fi
echo ""

# Check if fonts are downloaded
echo "🔤 Checking Fonts..."
echo "------------------------------"
if [ -f ".fonts/CrimsonPro.ttf" ] && [ -f ".fonts/Cardo-Regular.ttf" ]; then
    echo -e "${GREEN}✓${NC} Fonts are downloaded"
else
    echo -e "${YELLOW}⚠${NC} Fonts are NOT downloaded"
    echo "   Run: mise run assets:fonts"
fi
echo ""

# Check running services
echo "🚀 Checking Running Services..."
echo "------------------------------"
check_port 5173 "Frontend (SvelteKit)"
check_port 8787 "Backend API (Cloudflare Worker)"
check_port 3001 "Typst Service"
echo ""

# Check log files
echo "📝 Checking Log Files..."
echo "------------------------------"
if [ -d ".devcontainer/logs" ]; then
    echo -e "${GREEN}✓${NC} Logs directory exists"
    
    for service in frontend backend-api typst-service; do
        if [ -f ".devcontainer/logs/${service}.log" ]; then
            echo -e "${GREEN}✓${NC} ${service}.log exists"
        else
            echo -e "${YELLOW}⚠${NC} ${service}.log does NOT exist"
        fi
    done
else
    echo -e "${YELLOW}⚠${NC} Logs directory does NOT exist"
fi
echo ""

# Summary
echo "=================================================="
echo "Health check complete!"
echo ""
echo "💡 Tips:"
echo "   - To start all services: bash .devcontainer/start-services.sh"
echo "   - To stop all services: bash .devcontainer/stop-services.sh"
echo "   - To view logs: tail -f .devcontainer/logs/<service>.log"
echo "   - To see all mise tasks: mise tasks"
