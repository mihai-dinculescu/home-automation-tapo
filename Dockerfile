# Caching stage
FROM lukemathwalker/cargo-chef:latest-rust-1.61 as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM lukemathwalker/cargo-chef:latest-rust-1.61 as cacher

RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install pkg-config libssl-dev cmake

WORKDIR /app
RUN rustup component add rustfmt
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Builder stage
FROM rust:1.61 AS builder

RUN rustup component add rustfmt
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release

# Runtime stage
FROM debian:buster-slim AS runtime

RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install openssl

WORKDIR /app
COPY --from=builder /app/target/release/home-automation-tapo home-automation-tapo
COPY settings.yaml settings.yaml

ENTRYPOINT ["/app/home-automation-tapo"]
