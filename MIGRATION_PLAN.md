# Migration Plan: Shuttle to Railway (Docker)

## Overview

This document outlines the refactoring plan to migrate the crustyrustacean-dev-blog application from Shuttle.dev to Railway using Docker containers. The migration replaces Shuttle's managed infrastructure with direct configuration via environment variables.

## Current Shuttle Dependencies

The following Shuttle-specific dependencies and patterns are currently in use:

### Dependencies (Cargo.toml)
- `shuttle-axum = "0.57.0"`
- `shuttle-opendal = "0.57.0"`
- `shuttle-runtime = "0.57.0"`
- `shuttle-turso = "0.57.0"`
- `shuttle-common = "0.57.0"` (dev-dependency)

### Files Affected
1. `src/bin/main.rs` - Shuttle entry point with annotations
2. `src/lib/config.rs` - Uses `SecretStore` for configuration
3. `src/lib/service.rs` - Implements `shuttle_runtime::Service` trait
4. `tests/api/helpers.rs` - Uses Shuttle secret loading pattern
5. `Shuttle.toml` - Shuttle-specific build configuration

---

## Migration Tasks

### 1. Update Cargo.toml

**Remove:**
```toml
shuttle-axum = "0.57.0"
shuttle-opendal = "0.57.0"
shuttle-runtime = { version = "0.57.0", default-features = false }
shuttle-turso = "0.57.0"
shuttle-common = "0.57.0"  # dev-dependency
```

**Add/Modify:**
```toml
tokio = { version = "1.47.1", features = ["rt-multi-thread", "macros"] }
```

**Keep unchanged:**
- `libsql` - Direct Turso database access
- `opendal` - Direct S3 storage access
- `dotenvy` - Already present for .env loading

---

### 2. Refactor src/lib/config.rs

**Current Pattern:**
```rust
use shuttle_runtime::{CustomError, SecretStore};

impl TryFrom<&SecretStore> for AppConfig {
    type Error = CustomError;
    fn try_from(secrets: &SecretStore) -> Result<Self> {
        let jwt_secret = secrets.get("JWT_SECRET").ok_or_else(|| ...)?;
        // ...
    }
}
```

**New Pattern:**
```rust
use anyhow::{Result, anyhow};
use std::env;

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| anyhow!("Missing required environment variable: JWT_SECRET"))?;
        // ... similar for other required vars
    }
}
```

---

### 3. Refactor src/lib/service.rs

**Remove:**
```rust
use shuttle_runtime::Service;

#[shuttle_runtime::async_trait]
impl Service for AppService {
    async fn bind(mut self, addr: SocketAddr) -> Result<(), shuttle_runtime::Error> {
        // ...
    }
}
```

**Keep:**
- The `AppService` struct
- The `build_router()` method
- The `run_until_stopped()` method (for tests)
- Add a new `run()` method for production use

---

### 4. Refactor src/bin/main.rs

**Current Pattern:**
```rust
use shuttle_runtime::{CustomError, Error, SecretStore, Secrets};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_turso::Turso(...)] turso_client: libsql::Database,
    #[shuttle_opendal::Opendal(scheme = "s3")] operator: Operator,
    #[Secrets] secrets: SecretStore,
) -> Result<AppService, Error> {
    // ...
}
```

**New Pattern:**
```rust
use anyhow::Result;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file for local development
    dotenvy::dotenv().ok();

    // Initialize tracing
    let subscriber = get_subscriber(...);
    init_subscriber(subscriber);

    // Load configuration from environment
    let app_config = AppConfig::from_env()?;

    // Initialize Turso database connection
    let turso_url = env::var("TURSO_DATABASE_URL")?;
    let turso_token = env::var("TURSO_AUTH_TOKEN")?;
    let turso_client = libsql::Builder::new_remote(turso_url, turso_token)
        .build()
        .await?;
    let db = DatabaseConnection { db: Arc::new(turso_client) };
    db.run_migrations().await?;

    // Initialize S3 storage via OpenDAL
    let operator = create_s3_operator()?;
    let storage: Arc<dyn StorageBackend> = Arc::new(OpenDalStorage::new(operator));

    // Build application state and service
    let app_state = AppState::new(db, storage, app_config)?;
    let app_service = AppService::new(app_state);

    // Bind to port (Railway provides PORT env var)
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "8000".to_string()).parse()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    app_service.run_until_stopped(listener).await?;

    Ok(())
}

fn create_s3_operator() -> Result<Operator> {
    let mut builder = opendal::services::S3::default();
    builder
        .bucket(&env::var("S3_BUCKET")?)
        .region(&env::var("S3_REGION")?)
        .access_key_id(&env::var("S3_ACCESS_KEY_ID")?)
        .secret_access_key(&env::var("S3_SECRET_ACCESS_KEY")?);

    // Optional: Custom endpoint for S3-compatible services (R2, MinIO, etc.)
    if let Ok(endpoint) = env::var("S3_ENDPOINT") {
        builder.endpoint(&endpoint);
    }

    Ok(Operator::new(builder)?.finish())
}
```

---

### 5. Refactor tests/api/helpers.rs

**Remove:**
```rust
use shuttle_common::secrets::Secret;
use shuttle_runtime::SecretStore;

fn load_test_secret_store() -> Result<SecretStore> { ... }
```

**New Pattern:**
```rust
fn load_test_env() -> Result<()> {
    // Load from .env.test or Secrets.dev.toml (converted to env vars)
    dotenvy::from_filename(".env.test").or_else(|_| dotenvy::dotenv()).ok();
    Ok(())
}

// Use AppConfig::from_env() instead of TryFrom<&SecretStore>
```

---

### 6. Create Dockerfile

```dockerfile
# Build stage
FROM rust:1.83-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/crustyrustacean-dev-blog ./

# Copy static assets
COPY templates ./templates
COPY static ./static
COPY themes ./themes

# Railway provides PORT via environment variable
ENV PORT=8000
EXPOSE 8000

CMD ["./crustyrustacean-dev-blog"]
```

---

### 7. Delete Shuttle.toml

Remove this file as it's no longer needed for Railway deployment.

---

### 8. Create .env.example

```env
# Database (Turso)
TURSO_DATABASE_URL=libsql://your-database.turso.io
TURSO_AUTH_TOKEN=your-auth-token

# S3 Storage (Cloudflare R2, AWS S3, etc.)
S3_BUCKET=your-bucket-name
S3_REGION=auto
S3_ACCESS_KEY_ID=your-access-key
S3_SECRET_ACCESS_KEY=your-secret-key
S3_ENDPOINT=https://your-account-id.r2.cloudflarestorage.com  # Optional, for R2/MinIO

# Application Configuration
JWT_SECRET=your-jwt-secret-key
TEMPLATES_DIR=templates/**/*
ALLOWED_ORIGINS=https://yourdomain.com,http://localhost:8000

# Email (Mailtrap) - Optional
MAILTRAP_API_TOKEN=
MAILTRAP_SANDBOX_INBOX_ID=
MAILTRAP_SENDER_EMAIL=noreply@example.com
MAILTRAP_SENDER_NAME=CrustyRustacean Dev Blog

# Server
PORT=8000
```

---

## Environment Variables Summary

| Variable | Required | Description |
|----------|----------|-------------|
| `TURSO_DATABASE_URL` | Yes | Turso database URL |
| `TURSO_AUTH_TOKEN` | Yes | Turso authentication token |
| `S3_BUCKET` | Yes | S3/R2 bucket name |
| `S3_REGION` | Yes | S3 region (use "auto" for R2) |
| `S3_ACCESS_KEY_ID` | Yes | S3 access key |
| `S3_SECRET_ACCESS_KEY` | Yes | S3 secret key |
| `S3_ENDPOINT` | No | Custom S3 endpoint (for R2/MinIO) |
| `JWT_SECRET` | Yes | JWT signing secret |
| `TEMPLATES_DIR` | Yes | Template directory glob pattern |
| `ALLOWED_ORIGINS` | No | CORS allowed origins (comma-separated) |
| `MAILTRAP_API_TOKEN` | No | Mailtrap API token |
| `PORT` | No | Server port (default: 8000) |

---

## Railway Deployment Notes

1. **Automatic Dockerfile Detection**: Railway will detect the Dockerfile and build automatically
2. **Environment Variables**: Set all required variables in Railway dashboard
3. **PORT**: Railway automatically injects the `PORT` environment variable
4. **Health Check**: The `/health_check` endpoint is already implemented

---

## Testing Strategy

1. Create a `.env.test` file with test credentials
2. Run `cargo test` to verify all tests pass
3. Build Docker image locally: `docker build -t crustyrustacean-dev-blog .`
4. Test Docker image: `docker run --env-file .env -p 8000:8000 crustyrustacean-dev-blog`

---

## Risk Mitigation

- **Backwards Compatibility**: The `dotenvy` crate is already a dependency, minimizing changes
- **Test Coverage**: Existing tests will be updated to use environment variables
- **Gradual Migration**: Each component can be tested independently
