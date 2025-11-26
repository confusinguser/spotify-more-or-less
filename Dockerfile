# Unified Dockerfile for both frontend and backend
FROM node:20-slim AS frontend-builder
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
WORKDIR /app/frontend

# Copy package files and install dependencies
COPY web/package.json web/pnpm-lock.yaml* ./
RUN pnpm i

# Copy frontend source and build
COPY web/ .
RUN pnpm build

# Rust backend stage
FROM rust:slim AS backend-builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files first for better caching
COPY app/Cargo.toml app/Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release

# Remove the dummy artifacts and source
RUN rm -rf src target/release/deps/Spotify_More_Less* target/release/Spotify-More-Less*

# Copy the actual source code
COPY app/src ./src

# Force rebuild of the actual binary
RUN cargo build --release

# Final runtime stage combining both
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js for serving the frontend
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

WORKDIR /app

# Copy the Rust backend binary
COPY --from=backend-builder /app/target/release/Spotify-More-Less /app/spotify-more-less

# Copy the built frontend
COPY --from=frontend-builder /app/frontend/.next/standalone ./frontend/
COPY --from=frontend-builder /app/frontend/.next/static ./frontend/.next/static
COPY --from=frontend-builder /app/frontend/public ./frontend/public
COPY ./start.sh /app/start.sh
RUN chmod +x /app/start.sh

# Create data directory for persistent storage
RUN mkdir -p /app/data

# Set environment variables
ENV DATA_DIR=/app/data
ENV NODE_ENV=production

# Expose both ports (backend 8000, frontend 3000)
EXPOSE 3000 8000

CMD ["/app/start.sh"]
