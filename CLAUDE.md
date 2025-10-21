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
- **Tokio Runtime**: Async runtime with optimized feature flags (`rt`, `macros`, `rt-multi-thread`) for smaller binary size
- **Tracing**: Structured logging and telemetry with `tracing` and `tracing-subscriber`
- **Tower-HTTP**: HTTP middleware including request tracing
- **Configuration-driven**: Uses YAML configuration files with layered base/environment-specific approach
- **Secrecy**: Secure handling of sensitive data (passwords, connection strings) using `secrecy` crate with `SecretString`
- **Domain-Driven Design**: Type-driven development with domain types and validation
- **Unicode Segmentation**: Unicode-aware text processing with `unicode-segmentation` crate
- **Email Client**: HTTP-based email client using `reqwest` with TLS support for sending transactional emails
- **Validator**: Email validation using `validator` crate with `ValidateEmail` trait

### Module Organization
- `src/main.rs` - Application entry point with telemetry initialization
- `src/lib.rs` - Module declarations
- `src/startup.rs` - Server startup logic, route configuration, and HTTP tracing middleware
- `src/configurations.rs` - Configuration management and database settings
- `src/email_client.rs` - Email client for sending transactional emails
- `src/domain/` - Domain types and business logic with validation
  - `mod.rs` - Module exports and re-exports
  - `subscriber_name.rs` - Type-safe subscriber name with validation
  - `subscriber_email.rs` - Type-safe email with validation using `validator` crate
  - `new_subscriber.rs` - Domain model for new subscriptions
- `src/routes/` - HTTP route handlers
  - `health_check.rs` - Health check endpoint
  - `subscriptions.rs` - Newsletter subscription endpoint with database persistence
- `tests/health.rs` - Integration tests with isolated database setup

### Key Components

**Application Setup**: The app creates an Axum router with shared state:
- `GET /health` - Health check endpoint
- `POST /subscriptions` - Newsletter subscription endpoint with database persistence
- Shared state via Axum state management:
  - Database connection pool (`PgPool`)
  - Email client (`Arc<EmailClient>`) for sending transactional emails
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
- Email client settings:
  - `base_url` - Email service provider API endpoint
  - `sender_email` - Validated sender email address (parsed via `SubscriberEmail::parse`)
- Secure password handling using `SecretString` from the `secrecy` crate
- Passwords exposed only when needed via `.expose_secret()` within `PgConnectOptions`

**Domain Layer**: Type-driven development with domain types and validation:
- **Type Safety**: Domain types encapsulate business logic and enforce invariants at compile time
- **Parse, Don't Validate**: Uses constructor functions (`parse`) instead of validation functions
  - Constructor returns `Result<T, String>` for error handling
  - Once created, domain types guarantee their invariants are maintained
- **SubscriberName Type**: Validated subscriber name with constraints:
  - Must not be empty or whitespace-only
  - Maximum length: 256 graphemes (Unicode-aware using `unicode-segmentation` crate)
  - Forbidden characters: `/ ( ) " < > \ { }`
  - Uses newtype pattern (`SubscriberName(String)`) to prevent misuse
  - Implements `AsRef<str>` for ergonomic string access without exposing inner `String`
- **SubscriberEmail Type**: Validated email address with constraints:
  - Uses `validator` crate's `ValidateEmail` trait for RFC-compliant email validation
  - Returns `Result<SubscriberEmail, String>` from `parse` constructor
  - Uses newtype pattern (`SubscriberEmail(String)`) to prevent misuse
  - Implements `AsRef<str>` for ergonomic string access
- **NewSubscriber Model**: Domain model for subscription requests
  - Combines validated `SubscriberEmail` and `SubscriberName`
  - Used in route handlers to ensure only valid data reaches database layer
  - Implements `TryFrom<FormData>` for type-safe conversion from HTTP form data
- **Ownership and Borrowing**: Leverages Rust's ownership system
  - `parse` takes ownership of `String` to construct domain type
  - `AsRef<str>` allows borrowing the inner string without exposing mutation
  - Database queries use `.as_ref()` to access the validated string slice

**Email Client**: HTTP-based client for sending transactional emails:
- **Structure**: `EmailClient` struct with:
  - `http_client` - `reqwest::Client` for making HTTP requests
  - `base_url` - Email service provider API endpoint (Postmark in production, localhost in dev)
  - `sender` - Validated `SubscriberEmail` for the sender address
- **Features**:
  - Async email sending via `send_email` method
  - Type-safe recipient addresses using `SubscriberEmail` domain type
  - Support for both HTML and plain text content
  - Built on `reqwest` with TLS support (rustls-tls)
  - JSON serialization via `serde` for email request payloads
- **Integration**:
  - Configured via YAML settings in `EmailClientSettings`
  - Sender email validated at startup via `SubscriberEmail::parse`
  - Shared across routes via `Arc<EmailClient>` in Axum state
  - Email client initialized in both `main.rs` and test setup with validated sender
- **Testing**:
  - Unit tests using `wiremock` crate for HTTP mocking
  - Mock server simulates email service provider responses
  - Tests verify request dispatch to correct endpoint

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
- **Property-Based Testing**: Uses `quickcheck` and `fake` crates for:
  - Generating arbitrary test data
  - Testing domain invariants with randomized inputs
  - Email validation testing with generated valid emails via `SafeEmail` faker
  - Custom `Arbitrary` implementations for domain-specific test fixtures
- **HTTP Mocking**: Uses `wiremock` crate for testing HTTP clients:
  - Mock server creation for simulating external API responses
  - Request matching and verification
  - Testing email client without hitting real email service providers
- **Assertion Library**: Uses `assertables` crate for ergonomic test assertions (e.g., `assert_err!`)

---

## Documentation Update Log

### 2025-10-21 (Commits: 9120863..ead093b + staged changes)
**Major Changes:**
- **Email Client Completion**:
  - Implemented `send_email` method with HTTP POST request to email service
  - Added `SendEmailRequest` struct with `serde` serialization for JSON payloads
  - Email client now sends to `{base_url}/email` endpoint
  - Request includes sender, recipient, subject, HTML and text body
- **HTTP Mocking for Tests**:
  - Integrated `wiremock` crate (v0.6.5) for HTTP testing
  - Added unit tests for email client using mock servers
  - Tests verify request dispatching without hitting real email APIs
  - Mock server validates email sending behavior
- **Tokio Runtime Optimization**:
  - Changed from `features = ["full"]` to specific features: `rt`, `macros`, `rt-multi-thread`
  - Reduces binary size by excluding unused Tokio features
  - Maintains all required async runtime functionality
- **Configuration Consolidation**:
  - Removed root-level `configuration.yaml` file
  - All configuration now uses layered approach: `base.yaml` + environment-specific overlays
  - Email client settings added to both `base.yaml` and `production.yaml`
  - Production uses Postmark API (`https://api.postmarkapp.com`)
  - Local development uses localhost for email testing
- **Additional Dependencies**:
  - `assert-json-diff` (v2.0.2) - JSON comparison utilities
  - `deadpool` (v0.12.3) - Connection pool for wiremock
  - `futures` (v0.3.31) - Async utilities for wiremock
  - `h2` (v0.4.12) - HTTP/2 support for hyper
  - `tokio-util` (v0.7.16) - Additional Tokio utilities

### 2025-10-20 (Commits: 39b20de..1650ed3)
**Major Changes:**
- **Email Client Implementation**:
  - Added `src/email_client.rs` with `EmailClient` struct for sending transactional emails
  - Integrated `reqwest` crate with JSON and rustls-tls features for HTTP client
  - Email client configured via `EmailClientSettings` in configuration system
  - Sender email validated at startup using domain validation
  - Email client shared across routes via `Arc<EmailClient>` in Axum state
- **Enhanced Domain Layer**:
  - Added `SubscriberEmail` domain type with email validation
  - Implemented email validation using `validator` crate's `ValidateEmail` trait
  - Refactored domain layer into separate files (`subscriber_name.rs`, `subscriber_email.rs`, `new_subscriber.rs`)
  - Updated `NewSubscriber` to use both `SubscriberEmail` and `SubscriberName` domain types
  - Implemented `TryFrom<FormData>` trait for type-safe conversion from HTTP forms
- **Property-Based Testing**:
  - Integrated `quickcheck` and `quickcheck_macros` for property-based testing
  - Added `fake` crate for generating realistic test data
  - Implemented custom `Arbitrary` trait for `ValidEmailFixture` test fixture
  - Added property-based tests for email validation using generated data
- **Configuration Updates**:
  - Added `email_client` section to configuration system
  - Email client settings include `base_url` and `sender_email`
  - Sender email validated via `SubscriberEmail::parse` in configuration
  - Configuration supports local and production email service endpoints
- **Testing Improvements**:
  - Added `assertables` crate for ergonomic assertions
  - Enhanced test coverage with property-based testing for domain validation
  - Test setup updated to initialize email client for integration tests

### 2025-10-12 (Commit: 62d5327)
**Major Changes:**
- Introduced domain-driven design with type-safe domain layer (`src/domain.rs`)
- Added `SubscriberName` newtype with validation logic:
  - Unicode-aware length validation (max 256 graphemes)
  - Forbidden character filtering
  - Empty/whitespace validation
- Implemented "Parse, Don't Validate" pattern with constructor function
- Added `NewSubscriber` domain model for type-safe subscription handling
- Integrated `unicode-segmentation` crate for proper Unicode text handling
- Implemented `AsRef<str>` trait for ergonomic string access
- Updated subscription route to use domain types for validation
- Enhanced type safety by moving validation from route layer to domain layer

### 2025-10-12 (Commits: a73e6ec..eb4b339)
**Major Changes:**
- Added SSL/TLS support for PostgreSQL connections using `PgConnectOptions`
- Implemented environment-specific SSL configuration (required in production, preferred in local)
- Added DigitalOcean managed database integration with automatic credential injection
- Refactored database connection from connection strings to `PgConnectOptions` builder pattern
- Added statement logging at trace level for production debugging
- Configured runtime environment variable injection in `spec.yaml` for database credentials
- Enhanced configuration system with `require_ssl` flag for database settings