# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Build and Test
- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo run` - Start the development server (runs on port 8000)

### Code Quality
- `cargo fmt --check` - Check code formatting
- `cargo fmt` - Format code
- `cargo clippy -- -D warnings` - Run clippy linter with warnings as errors

### Database Management
- `./scripts/init_db.sh` - Initialize and migrate the PostgreSQL database
- `SKIP_DOCKER=true ./scripts/init_db.sh` - Initialize database without Docker (assumes existing PostgreSQL)
- `sqlx migrate run` - Run database migrations
- `sqlx database create` - Create the database
- `cargo sqlx prepare` - Generate offline query metadata for SQLx (enables compilation without DB connection)

### SQL Linting
- `sqruff lint .` - Lint SQL files
- `sqruff fix .` - Auto-fix SQL formatting issues

## Architecture Overview

This is a Rust web application built with Axum for creating a newsletter subscription service. The project follows a modular architecture:

### Core Structure
- **Axum Framework**: Web server framework with async/await support
- **SQLx**: Type-safe SQL toolkit for PostgreSQL integration with connection pooling
- **UUID**: Unique identifier generation with `uuid` crate (v4 UUIDs)
- **Tracing**: Structured logging and telemetry with `tracing` and `tracing-subscriber`
- **Tower-HTTP**: HTTP middleware including request tracing
- **Configuration-driven**: Uses YAML configuration files
- **Secrecy**: Secure handling of sensitive data (passwords, connection strings) using `secrecy` crate with `SecretString`

### Module Organization
- `src/main.rs` - Application entry point with telemetry initialization
- `src/lib.rs` - Module declarations
- `src/startup.rs` - Server startup logic, route configuration, and HTTP tracing middleware
- `src/configurations.rs` - Configuration management and database settings
- `src/routes/` - HTTP route handlers
  - `health_check.rs` - Health check endpoint
  - `subscriptions.rs` - Newsletter subscription endpoint with database persistence
- `tests/health.rs` - Integration tests with isolated database setup

### Key Components

**Application Setup**: The app creates an Axum router with shared database state:
- `GET /health` - Health check endpoint
- `POST /subscriptions` - Newsletter subscription endpoint with database persistence
- Database connection pool shared across all routes via Axum state
- HTTP request tracing middleware for observability

**Telemetry**: Structured logging and observability features:
- Tracing subscriber with environment-based log level configuration (defaults to "info")
- Generic sink configuration using higher-rank trait bounds (HRTBs) with `MakeWriter` trait
  - Allows flexible output destinations (stdout, files, test buffers, etc.)
  - Uses `for<'a> MakeWriter<'a>` to ensure the writer works with any lifetime
- Bunyan-formatted JSON logging for structured output
- HTTP request/response tracing via Tower-HTTP TraceLayer
- Structured logging using `tracing::info!` instead of `println!`
- `#[tracing::instrument]` proc macro for automatic span creation
  - Eliminates manual span guards and `.instrument()` boilerplate
  - Automatically tracks function entry/exit and arguments
  - Supports custom span names, field skipping, and computed fields
  - Example: `#[tracing::instrument(name="...", skip(...), fields(...))]`

**Database Integration**: Uses SQLx for PostgreSQL operations with:
- Connection pooling via `PgPool` with shared state across routes
- Type-safe queries with compile-time verification
- Database migrations managed by `sqlx migrate`
- Isolated test databases for each test run using random UUIDs
- SSL/TLS support with `PgConnectOptions` for secure connections
  - Production: SSL required (`PgSslMode::Require`)
  - Local development: SSL preferred but not required (`PgSslMode::Prefer`)
- Statement logging at trace level for debugging (production only)

**Configuration**: Multi-layered YAML-based configuration system with:
- Environment-based configuration selection via `APP_ENVIRONMENT` variable (defaults to "local")
- Configuration structure:
  - `configuration/base.yaml` - Base configuration with common settings
  - `configuration/local.yaml` - Local development overrides (host: 127.0.0.1, SSL not required)
  - `configuration/production.yaml` - Production overrides (host: 0.0.0.0, SSL required)
- Environment variable injection support with `APP_` prefix (e.g., `APP_DATABASE__HOST`)
  - Uses double underscore (`__`) as nested field separator
  - Example: `APP_DATABASE__HOST=db.example.com` overrides `database.host`
- Database connection settings:
  - Host, port, username, password, database name
  - `require_ssl` flag for SSL/TLS enforcement
  - Uses `PgConnectOptions` builder pattern instead of connection strings
- Application port and host configuration
- Secure password handling using `SecretString` from the `secrecy` crate
- Passwords exposed only when needed via `.expose_secret()` within `PgConnectOptions`

### Database Schema
The application manages newsletter subscriptions with a `subscriptions` table containing:
- `id` (UUID)
- `email` (String)
- `name` (String) 
- `subscribed_at` (timestamp)

### Development Environment
- PostgreSQL database required (default port 5433)
- Docker support for database setup via `init_db.sh`

### Docker Deployment
Multi-stage Dockerfile optimized for production:
- **Base Stage**: Rust 1.90.0 with cargo-chef for dependency caching
- **Build Optimizations**:
  - Mold linker for faster builds
  - Symbol stripping for smaller binary size
  - Cargo-chef for Docker layer caching of dependencies
  - SQLx offline mode enabled for compilation without database connection
- **Runtime Stage**: Ubuntu 24.04 minimal image
  - CA certificates for HTTPS support
  - Aggressive cleanup to minimize image size
  - Configuration files copied for environment-based settings
  - `APP_ENVIRONMENT=production` set by default

### DigitalOcean Deployment
Configured for deployment via App Platform (`spec.yaml`):
- **Application**:
  - Auto-deploy from GitHub main branch
  - Health check endpoint at `/health`
  - HTTP port 8000
  - Instance: basic-xxs (1 node)
  - Region: San Francisco (sfo)
- **Managed Database**:
  - PostgreSQL 17
  - Development size (db-s-dev-database)
  - Single node configuration
- **Environment Variables**: Automatically injected at runtime
  - Database credentials: `APP_DATABASE__USERNAME`, `APP_DATABASE__PASSWORD`
  - Connection details: `APP_DATABASE__HOST`, `APP_DATABASE__PORT`, `APP_DATABASE__DATABASE_NAME`
  - All values sourced from managed database service (e.g., `${newsletter.USERNAME}`)
  - SSL/TLS enforced for all database connections in production

### CI/CD Pipeline
GitHub Actions workflow with:
- PostgreSQL 17 service container for integration tests
- Code formatting checks (`cargo fmt`)
- Linting with Clippy (`cargo clippy -- -D warnings`)
- SQL linting with `sqruff`
- Database migration automation via `init_db.sh`
- SQLx offline mode verification (`cargo sqlx prepare --check`)
- Automated testing on push and pull requests
- Isolated test databases created automatically for each test run

### Testing Strategy
- **Integration Tests**: Full database integration with isolated test databases
- **Test Database Setup**: Each test creates a unique database using UUID naming
- **Automatic Cleanup**: Test databases are isolated and don't interfere with each other
- **Migration Testing**: Database migrations are run automatically in test setup

---

## Documentation Update Log

### 2025-10-12 (Commits: a73e6ec..eb4b339)
**Major Changes:**
- Added SSL/TLS support for PostgreSQL connections using `PgConnectOptions`
- Implemented environment-specific SSL configuration (required in production, preferred in local)
- Added DigitalOcean managed database integration with automatic credential injection
- Refactored database connection from connection strings to `PgConnectOptions` builder pattern
- Added statement logging at trace level for production debugging
- Configured runtime environment variable injection in `spec.yaml` for database credentials
- Enhanced configuration system with `require_ssl` flag for database settings