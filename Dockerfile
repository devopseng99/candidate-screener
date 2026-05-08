FROM rust:1.76-bookworm AS builder

WORKDIR /build

# Cache dependencies layer
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release 2>/dev/null; rm -rf src

COPY src/ ./src/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r screener && useradd -r -g screener -s /sbin/nologin screener

WORKDIR /app

COPY --from=builder /build/target/release/candidate-screener /usr/local/bin/candidate-screener

RUN mkdir -p /app/config /app/data && chown -R screener:screener /app

USER screener

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/candidate-screener"]
CMD ["serve", "--port", "8080"]
