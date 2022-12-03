# Compute the recipe file
FROM lukemathwalker/cargo-chef:latest-rust-1.59.0 as chef
WORKDIR /app
FROM chef as planner
COPY . .

# Compute a lockfile for our project
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage
FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim as runtime
WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder env to runtime
COPY --from=builder /app/target/release/z2p z2p
# Need configuration at runtime
COPY config config
ENV APP_ENVIRONMENT production

ENTRYPOINT ["./z2p"]
