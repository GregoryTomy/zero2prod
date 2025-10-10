# Base stage
FROM rust:1.90.0 AS chef
RUN cargo install --locked cargo-chef
RUN apt update && apt install -y mold
ENV RUSTFLAGS="-C strip=symbols -C link-arg=-fuse-ld=mold"
WORKDIR /app


# Planner stage
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin zero2prod

# Runtime stage
FROM ubuntu:24.04 AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean \
    && rm -rf /var/cache/apt/* \
    && rm -rf /usr/share/doc/* \
    && rm -rf /usr/share/man/* \
    && rm -rf /tmp/*

COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT [ "./zero2prod" ]
