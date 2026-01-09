# Use Ubuntu as base for better compatibility
FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    # Build essentials
    build-essential \
    pkg-config \
    libssl-dev \
    # Curl and wget for downloads
    curl \
    wget \
    # Git
    git \
    # For Rust compilation
    gcc \
    g++ \
    # For wasm32 target
    lld \
    clang \
    # Additional utilities
    unzip \
    jq \
    ca-certificates \
    gnupg \
    # For Typst
    fontconfig \
    && rm -rf /var/lib/apt/lists/*

# Install Rust and Cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/usr/local/cargo/bin:${PATH}"
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup

# Install Bun
RUN curl -fsSL https://bun.sh/install | bash
ENV BUN_INSTALL="/root/.bun"
ENV PATH="${BUN_INSTALL}/bin:${PATH}"

# Install Node.js LTS (via mise will handle this, but having a base version helps)
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install mise (will be configured via feature, but pre-install for faster setup)
RUN curl https://mise.run | sh
ENV PATH="/root/.local/bin:${PATH}"

# Pre-install wasm32 target for Rust
RUN rustup target add wasm32-unknown-unknown

# Install worker-build for Cloudflare Workers
RUN cargo install worker-build

# Install Typst
RUN curl -fsSL https://github.com/typst/typst/releases/download/v0.12.0/typst-x86_64-unknown-linux-musl.tar.xz | \
    tar -xJ -C /usr/local/bin --strip-components=1 && \
    chmod +x /usr/local/bin/typst

# Create workspace directory
RUN mkdir -p /workspace

# Set working directory
WORKDIR /workspace

# Switch to vscode user (created by base image)
USER vscode

# Install Bun for vscode user
RUN curl -fsSL https://bun.sh/install | bash
ENV BUN_INSTALL="/home/vscode/.bun"
ENV PATH="${BUN_INSTALL}/bin:${PATH}"

# Install mise for vscode user
RUN curl https://mise.run | sh
ENV PATH="/home/vscode/.local/bin:${PATH}"

# Configure mise directory
RUN mkdir -p /home/vscode/.local/share/mise

# Reset to root for final setup
USER root

# Ensure vscode user has access to cargo
RUN chown -R vscode:vscode /usr/local/cargo /usr/local/rustup

# Switch back to vscode user
USER vscode

# Set environment variables for vscode user
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH="/usr/local/cargo/bin:${PATH}"

CMD ["/bin/bash"]
