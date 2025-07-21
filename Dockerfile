# Stage 1: Build
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /usr/src/app
RUN cargo new adobe_pdf_extractor
WORKDIR /usr/src/app/adobe_pdf_extractor

# Cache dependencies
COPY Cargo.toml .
RUN cargo build --release

# Copy source and rebuild
COPY src ./src
RUN sed -i 's/^version = .*/version = "0.1.0"/' Cargo.toml && \
    cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
COPY --from=builder /usr/src/app/adobe_pdf_extractor/target/release/adobe_pdf_extractor /usr/local/bin/
WORKDIR /app
ENTRYPOINT ["adobe_pdf_extractor", "--input", "/app/input/sample.pdf", "--output", "/app/output/sample.json"]