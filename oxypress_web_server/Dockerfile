# Stage 1: Compute dependency recipe
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cache & build statically linked binary

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this layer is cached unless Cargo.toml/Cargo.lock change

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl -p oxypress_web_server

# Stage 3: Minimal production runtime
FROM gcr.io/distroless/static-debian12 AS runtime
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/oxypress_web_server /app/server

# Copy client static web assets (if Actix serves your Yew bundle)
# COPY --from=client-builder /app/oxypress_client_web/dist /app/static

EXPOSE 8080 8081
USER nonroot:nonroot

CMD ["/app/server"]
