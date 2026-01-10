# Use Ubuntu as base for better compatibility
FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    wget \
    git \
    gcc \
    g++ \
    lld \
    clang \
    unzip \
    jq \
    ca-certificates \
    gnupg \
    fontconfig \
    && rm -rf /var/lib/apt/lists/*

# Install Rust and Cargo as root
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
    && . /root/.cargo/env \
    && rustup target add wasm32-unknown-unknown \
    && cargo install worker-build

# Install Bun as root
RUN curl -fsSL https://bun.sh/install | bash

# Install Node.js LTS
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install wrangler globally
RUN npm install -g wrangler

# Install mise as root
RUN curl https://mise.run | sh

# Install Typst
RUN curl -fsSL https://github.com/typst/typst/releases/download/v0.12.0/typst-x86_64-unknown-linux-musl.tar.xz | \
    tar -xJ -C /usr/local/bin --strip-components=1 && \
    chmod +x /usr/local/bin/typst

# Create workspace directory
RUN mkdir -p /workspace

# Create workspace directory and mise cache directories
RUN mkdir -p /workspace/.mise/cache /workspace/.mise/installs /workspace/.mise/downloads

# Give vscode user access to all installed tools and workspace
RUN chown -R vscode:vscode /root/.cargo /root/.rustup /root/.bun /root/.local /workspace

# Switch to vscode user
USER vscode

# Set up environment for vscode user
ENV CARGO_HOME=/root/.cargo
ENV RUSTUP_HOME=/root/.rustup
ENV BUN_INSTALL=/root/.bun
ENV PATH="/root/.cargo/bin:/root/.bun/bin:/root/.local/bin:${PATH}"

# Install mise for vscode user (in their home directory)
RUN curl https://mise.run | sh
ENV PATH="/home/vscode/.local/bin:${PATH}"

# Create mise config directory
RUN mkdir -p /home/vscode/.local/share/mise

# Set working directory
WORKDIR /workspace

CMD ["/bin/bash"]
