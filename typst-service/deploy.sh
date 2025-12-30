#!/bin/bash

# Typst Service Deployment Script for Fedora VM
# Run this script on your VM to set up the Typst compilation service

set -e

echo "🚀 Deploying Typst Compilation Service"
echo "======================================="

# Configuration
SERVICE_DIR="$HOME/typst-service"
SERVICE_NAME="typst-service"
NGINX_CONF="/etc/nginx/conf.d/typst.ifkash.dev.conf"

# Check if running on the VM
if [ ! -f /etc/fedora-release ]; then
    echo "⚠️  Warning: This script is designed for Fedora. Proceed anyway? (y/n)"
    read -r response
    if [ "$response" != "y" ]; then
        exit 1
    fi
fi

# Step 1: Install Bun if not installed
echo ""
echo "📦 Step 1: Checking Bun installation..."
if ! command -v bun &> /dev/null; then
    echo "Installing Bun..."
    curl -fsSL https://bun.sh/install | bash
    export PATH="$HOME/.bun/bin:$PATH"
else
    echo "✓ Bun already installed"
fi

# Step 2: Install Typst if not installed
echo ""
echo "📦 Step 2: Checking Typst installation..."
if ! command -v typst &> /dev/null; then
    echo "Installing Typst..."
    curl -fsSL https://typst.app/install.sh | sh
else
    echo "✓ Typst already installed"
fi

# Step 3: Create service directory
echo ""
echo "📁 Step 3: Setting up service directory..."
mkdir -p "$SERVICE_DIR"
cd "$SERVICE_DIR"

# Step 4: Copy fonts
echo ""
echo "🔤 Step 4: Setting up fonts..."
if [ -d "$HOME/.fonts" ]; then
    cp -r "$HOME/.fonts" "$SERVICE_DIR/fonts"
    echo "✓ Fonts copied from ~/.fonts"
elif [ -d "/usr/share/fonts" ]; then
    mkdir -p "$SERVICE_DIR/fonts"
    echo "⚠️  Using system fonts. Consider adding custom fonts to $SERVICE_DIR/fonts"
else
    echo "❌ No fonts found. Please add fonts to $SERVICE_DIR/fonts"
    exit 1
fi

# Step 5: Copy service files
echo ""
echo "📄 Step 5: Copying service files..."
echo "Please ensure these files are in the current directory:"
echo "  - index.ts"
echo "  - package.json"
echo "  - typst-service.service"
echo "  - nginx-typst.conf"
echo ""
echo "Press Enter when files are ready..."
read -r

# Step 6: Install dependencies
echo ""
echo "📦 Step 6: Installing dependencies..."
bun install

# Step 7: Test the service
echo ""
echo "🧪 Step 7: Testing service..."
timeout 5 bun run index.ts &
sleep 2
if curl -s http://localhost:3001/health | grep -q "ok"; then
    echo "✓ Service test successful"
    pkill -f "bun run index.ts" || true
else
    echo "❌ Service test failed"
    pkill -f "bun run index.ts" || true
    exit 1
fi

# Step 8: Install systemd service
echo ""
echo "⚙️  Step 8: Installing systemd service..."
sudo cp typst-service.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable $SERVICE_NAME
sudo systemctl start $SERVICE_NAME

# Check service status
if sudo systemctl is-active --quiet $SERVICE_NAME; then
    echo "✓ Service started successfully"
else
    echo "❌ Service failed to start. Check logs with: sudo journalctl -u $SERVICE_NAME"
    exit 1
fi

# Step 9: Configure Nginx
echo ""
echo "🌐 Step 9: Configuring Nginx..."
if command -v nginx &> /dev/null; then
    sudo cp nginx-typst.conf "$NGINX_CONF"
    sudo nginx -t
    if [ $? -eq 0 ]; then
        sudo systemctl reload nginx
        echo "✓ Nginx configured successfully"
    else
        echo "❌ Nginx configuration test failed"
        exit 1
    fi
else
    echo "⚠️  Nginx not found. Please install and configure manually."
    echo "   Configuration file saved to: nginx-typst.conf"
fi

# Step 10: Configure firewall
echo ""
echo "🔥 Step 10: Configuring firewall..."
if command -v firewall-cmd &> /dev/null; then
    sudo firewall-cmd --permanent --add-service=http
    sudo firewall-cmd --permanent --add-service=https
    sudo firewall-cmd --reload
    echo "✓ Firewall configured"
else
    echo "⚠️  firewalld not found. Ensure ports 80 and 443 are open."
fi

# Final status
echo ""
echo "✅ Deployment Complete!"
echo "======================"
echo ""
echo "Service Status:"
sudo systemctl status $SERVICE_NAME --no-pager | head -n 10
echo ""
echo "Test the service:"
echo "  curl https://typst.ifkash.dev/health"
echo ""
echo "View logs:"
echo "  sudo journalctl -u $SERVICE_NAME -f"
echo ""
echo "Restart service:"
echo "  sudo systemctl restart $SERVICE_NAME"
