FROM oven/bun:1-alpine

# Install Typst
RUN apk add --no-cache curl && \
    curl -fsSL https://github.com/typst/typst/releases/download/v0.12.0/typst-x86_64-unknown-linux-musl.tar.xz | \
    tar -xJ -C /usr/local/bin --strip-components=1 && \
    chmod +x /usr/local/bin/typst && \
    apk del curl

# Create app directory
WORKDIR /app

# Copy package files
COPY package.json ./

# Install dependencies (if any in future)
RUN bun install

# Copy application code
COPY index.ts ./

# Create fonts directory
RUN mkdir -p /app/fonts

# Copy fonts (will be mounted as volume)
# COPY fonts/ /app/fonts/

# Expose port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3001/health || exit 1

# Run the service
CMD ["bun", "run", "index.ts"]
