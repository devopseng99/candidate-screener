FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl && \
    apt-get update && apt-get install -y musl-tools && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache dependencies layer
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null; rm -rf src

COPY src/ ./src/
RUN touch src/main.rs && cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.21 AS runtime

RUN apk add --no-cache ca-certificates tzdata && \
    addgroup -S screener && adduser -S -G screener -s /sbin/nologin screener

WORKDIR /app

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/candidate-screener /usr/local/bin/candidate-screener

RUN mkdir -p /app/config /app/data && chown -R screener:screener /app

USER screener

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/candidate-screener"]
CMD ["serve", "--port", "8080"]
