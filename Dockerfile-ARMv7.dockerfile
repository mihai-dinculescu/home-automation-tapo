# Builder stage
FROM rust:1.61 AS builder

RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install pkg-config libssl-dev cmake

WORKDIR /app
COPY . .
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
