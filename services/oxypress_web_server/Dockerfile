# Stage 1: Compute dependency recipe
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl wasm32-unknown-unknown
RUN cargo install trunk --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS web-builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --package oxypress_web_server --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl -p oxypress_web_server

FROM chef AS client-builder
COPY --from=chef /usr/local/cargo/bin/trunk /usr/local/cargo/bin/trunk
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --package oxypress_web_client --target wasm32-unknown-unknown --recipe-path recipe.json
COPY ./Cargo.lock  ./crates/oxypress_web_client/Cargo.lock
COPY crates/oxypress_core ./crates/oxypress_core
COPY crates/oxypress_web_client ./crates/oxypress_web_client
WORKDIR /app/crates/oxypress_web_client
RUN trunk build

FROM gcr.io/distroless/static-debian13:debug AS dev
WORKDIR /oxypress
# Copy binary from builder
COPY --from=web-builder /app/target/x86_64-unknown-linux-musl/release/oxypress_web_server /oxypress/server
COPY --from=web-builder /app/services/oxypress_web_server/resources /oxypress/resources
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
