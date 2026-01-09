#!/bin/bash
set -e

echo "🎬 Starting all services..."

# Activate mise
eval "$(mise activate bash)"

# Create log directory
mkdir -p /workspace/.devcontainer/logs

# Function to start a service in the background
start_service() {
    local name=$1
    local command=$2
    local log_file="/workspace/.devcontainer/logs/${name}.log"
    
    echo "▶️  Starting ${name}..."
    nohup bash -c "$command" > "$log_file" 2>&1 &
    echo $! > "/workspace/.devcontainer/logs/${name}.pid"
    echo "   Logs: ${log_file}"
}

# Start Typst Service
start_service "typst-service" "cd /workspace/typst-service && bun run dev"

# Wait a bit for typst service to start
sleep 2

# Start Backend API (Cloudflare Worker)
start_service "backend-api" "cd /workspace/api && mise run api:dev"

# Wait a bit for backend to start
sleep 2

# Start Frontend (SvelteKit)
start_service "frontend" "cd /workspace && mise run web:dev"

echo ""
echo "✅ All services started!"
echo ""
echo "📊 Service Status:"
echo "   - Frontend (SvelteKit):     http://localhost:5173"
echo "   - Backend API (Worker):     http://localhost:8787"
echo "   - Typst Service:            http://localhost:3001"
echo ""
echo "📝 Logs are available in .devcontainer/logs/"
echo ""
echo "To stop services, run: bash .devcontainer/stop-services.sh"
