FROM rust:1.87-bookworm as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

RUN cargo build --release --bin chat_platform

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/chat_platform .

RUN mkdir -p /app/data/files /app/logs

COPY .env.example .env

EXPOSE 8080

CMD ["./chat_platform"]
