#!/bin/bash

# Typst Service Docker Deployment Script
# Run this on your VM to deploy the Typst service with Docker Compose

set -e

echo "🐳 Deploying Typst Service with Docker"
echo "======================================="

# Configuration
SERVICE_DIR="$HOME/typst-service"
FONTS_DIR="$SERVICE_DIR/fonts"

# Step 1: Check Docker and Docker Compose
echo ""
echo "📦 Step 1: Checking Docker installation..."
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first."
    exit 1
fi

if ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose not found. Please install Docker Compose first."
    exit 1
fi

echo "✓ Docker and Docker Compose are installed"

# Step 2: Create service directory
echo ""
echo "📁 Step 2: Setting up service directory..."
mkdir -p "$SERVICE_DIR"
cd "$SERVICE_DIR"

# Step 3: Set up fonts
echo ""
echo "🔤 Step 3: Setting up fonts..."
if [ ! -d "$FONTS_DIR" ] || [ -z "$(ls -A $FONTS_DIR)" ]; then
    echo "Creating fonts directory..."
    mkdir -p "$FONTS_DIR"
    
    # Download common fonts
    echo "Downloading fonts..."
    curl -L -o "$FONTS_DIR/CrimsonPro.ttf" "https://github.com/google/fonts/raw/main/ofl/crimsonpro/CrimsonPro[wght].ttf"
    curl -L -o "$FONTS_DIR/Cardo-Regular.ttf" "https://github.com/google/fonts/raw/main/ofl/cardo/Cardo-Regular.ttf"
    curl -L -o "$FONTS_DIR/Cardo-Bold.ttf" "https://github.com/google/fonts/raw/main/ofl/cardo/Cardo-Bold.ttf"
    
    echo "✓ Fonts downloaded"
else
    echo "✓ Fonts directory already exists"
fi

# Step 4: Copy service files
echo ""
echo "📄 Step 4: Copying service files..."
echo "Please ensure these files are in $SERVICE_DIR:"
echo "  - Dockerfile"
echo "  - docker-compose.yml"
echo "  - index.ts"
echo "  - package.json"
echo ""
echo "Press Enter when files are ready..."
read -r

# Step 5: Create web_network if it doesn't exist
echo ""
echo "🌐 Step 5: Setting up Docker network..."
if ! docker network inspect web_network &> /dev/null; then
    echo "Creating web_network..."
    docker network create web_network
    echo "✓ Network created"
else
    echo "✓ Network already exists"
fi

# Step 6: Build and start the service
echo ""
echo "🔨 Step 6: Building Docker image..."
docker compose build

echo ""
echo "🚀 Step 7: Starting service..."
docker compose up -d

# Step 8: Wait for service to be healthy
echo ""
echo "⏳ Step 8: Waiting for service to be healthy..."
sleep 5

if docker compose ps | grep -q "healthy"; then
    echo "✓ Service is healthy"
elif docker compose ps | grep -q "running"; then
    echo "⚠️  Service is running but health check not yet passed"
else
    echo "❌ Service failed to start. Check logs with: docker compose logs"
    exit 1
fi

# Step 9: Test the service
echo ""
echo "🧪 Step 9: Testing service..."
if curl -s http://localhost:3001/health | grep -q "ok"; then
    echo "✓ Service test successful"
else
    echo "❌ Service test failed"
    docker compose logs
    exit 1
fi

# Step 10: Update Caddyfile
echo ""
echo "📝 Step 10: Updating Caddyfile..."
CADDY_DIR="$HOME/caddy"  # Adjust this path if your Caddy config is elsewhere

if [ -f "$CADDY_DIR/Caddyfile" ]; then
    # Check if typst.ifkash.dev already exists
    if grep -q "typst.ifkash.dev" "$CADDY_DIR/Caddyfile"; then
        echo "⚠️  typst.ifkash.dev already exists in Caddyfile"
    else
        echo "Adding typst.ifkash.dev to Caddyfile..."
        cat >> "$CADDY_DIR/Caddyfile" << 'EOF'

typst.ifkash.dev {
  reverse_proxy typst-service:3001
}
EOF
        echo "✓ Caddyfile updated"
        
        # Reload Caddy
        echo "Reloading Caddy..."
        cd "$CADDY_DIR"
        docker compose exec caddy caddy reload --config /etc/caddy/Caddyfile
        echo "✓ Caddy reloaded"
    fi
else
    echo "⚠️  Caddyfile not found at $CADDY_DIR/Caddyfile"
    echo "   Please add this to your Caddyfile manually:"
    echo ""
    echo "typst.ifkash.dev {"
    echo "  reverse_proxy typst-service:3001"
    echo "}"
fi

# Final status
echo ""
echo "✅ Deployment Complete!"
echo "======================"
echo ""
echo "Service Status:"
docker compose ps
echo ""
echo "Test the service:"
echo "  curl http://localhost:3001/health"
echo "  curl https://typst.ifkash.dev/health"
echo ""
echo "View logs:"
echo "  docker compose logs -f"
echo ""
echo "Restart service:"
echo "  docker compose restart"
echo ""
echo "Stop service:"
echo "  docker compose down"
