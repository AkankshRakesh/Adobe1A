# Build stage
FROM rust:1.75 AS builder

WORKDIR /app
COPY Cargo.toml .
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/adobe1a /usr/local/bin/
WORKDIR /app
CMD ["adobe1a"]