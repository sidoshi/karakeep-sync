# Multi-stage build for lean Docker image
FROM rust:1.88-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build dependencies (this is the cached layer)
RUN cargo build --release --bin karakeep-sync

# Runtime stage - using distroless for minimal size
FROM gcr.io/distroless/cc-debian12

# Copy the binary from builder stage
COPY --from=builder /app/target/release/karakeep-sync /usr/local/bin/karakeep-sync

# Create non-root user (distroless already provides this)
USER nonroot:nonroot

# Set the binary as entrypoint
ENTRYPOINT ["karakeep-sync"]
