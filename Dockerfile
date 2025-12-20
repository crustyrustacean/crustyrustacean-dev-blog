# Build stage
FROM rust:1.83-bookworm AS builder

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src/bin src/lib && \
    echo "fn main() {}" > src/bin/main.rs && \
    echo "" > src/lib/lib.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Touch main.rs to ensure it's rebuilt
RUN touch src/bin/main.rs

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install required runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/crustyrustacean-dev-blog ./

# Copy static assets and templates
COPY templates ./templates
COPY static ./static
COPY themes ./themes

# Railway provides PORT via environment variable
ENV PORT=8000
EXPOSE 8000

# Run the application
CMD ["./crustyrustacean-dev-blog"]
