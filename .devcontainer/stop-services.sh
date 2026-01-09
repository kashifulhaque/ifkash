#!/bin/bash
set -e

echo "🛑 Stopping all services..."

# Function to stop a service
stop_service() {
    local name=$1
    local pid_file="/workspace/.devcontainer/logs/${name}.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if ps -p $pid > /dev/null 2>&1; then
            echo "   Stopping ${name} (PID: ${pid})..."
            kill $pid 2>/dev/null || true
            # Wait for process to stop
            sleep 1
            # Force kill if still running
            if ps -p $pid > /dev/null 2>&1; then
                kill -9 $pid 2>/dev/null || true
            fi
        fi
        rm "$pid_file"
    else
        echo "   ${name} is not running (no PID file found)"
    fi
}

# Stop all services
stop_service "frontend"
stop_service "backend-api"
stop_service "typst-service"

echo ""
echo "✅ All services stopped!"
