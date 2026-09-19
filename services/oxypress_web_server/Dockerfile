# Stage 1: Compute dependency recipe
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl wasm32-unknown-unknown
RUN cargo install trunk --locked


# Stage 1: build yew front-end for the oxypres
FROM chef AS client-builder
WORKDIR /app
COPY . .
WORKDIR /app/crates/oxypress_web_client
COPY --from=chef /usr/local/cargo/bin/trunk /usr/local/cargo/bin/trunk
RUN trunk build


FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Cache & build statically linked binary

FROM chef AS web-builder
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this layer is cached unless Cargo.toml/Cargo.lock change
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl -p oxypress_web_server



FROM gcr.io/distroless/static-debian13:debug AS dev
WORKDIR /oxypress
# Copy binary from builder
COPY --from=web-builder /app/target/x86_64-unknown-linux-musl/release/oxypress_web_server /oxypress/server
COPY --from=client-builder /app/crates/oxypress_web_client/dist /oxypress/resources/static
# Use auto-generated self-signed certs to run locally 
COPY ./certs/dev/ /etc/ssl/certs

EXPOSE 8081 8081
USER nonroot:nonroot
ENTRYPOINT ["/oxypress/server"]

FROM gcr.io/distroless/static-debian13 AS prod
WORKDIR /oxypress
# Copy binary from builder
COPY --from=web-builder /app/target/x86_64-unknown-linux-musl/release/oxypress_web_server /oxypress/server
COPY --from=client-builder /app/crates/oxypress_web_client/dist /oxypress/resources/static

EXPOSE 8081 8081
USER nonroot:nonroot
ENTRYPOINT ["/oxypress/server"]
