# Build stage
FROM rust:1.80 AS builder

WORKDIR /app

# Copy source files first
COPY Cargo.toml ./
COPY src ./src

# Build the application directly
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install required packages for PDF processing
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libc6 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/adobe1a /usr/local/bin/

WORKDIR /app

# Create necessary directories
RUN mkdir -p input output

# Copy the processing script
COPY process_all_pdfs.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/process_all_pdfs.sh

# Set the default command to process all PDFs
CMD ["/usr/local/bin/process_all_pdfs.sh"]