# Builder stage
FROM rust:1.90.0 AS builder

WORKDIR /app
RUN apt update && apt install -y mold
COPY . .
ENV SQLX_OFFLINE=true
ENV RUSTFLAGS="-C strip=symbols"
RUN cargo build --release

# Runtime stage
FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app

COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT [ "./targetrelease/zero2prod" ]
