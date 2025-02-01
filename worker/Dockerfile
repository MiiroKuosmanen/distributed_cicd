# Use Rust as the base image
FROM rust:1.84 AS builder

# Set working directory
WORKDIR /app

# Copy the Rust project files
COPY . .

# Build the worker binary
RUN cargo build --release

# Create a minimal runtime image
FROM ubuntu:22.04

# Set working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/worker /app/worker

# Expose the application port (if needed)
EXPOSE 5001 5002

# Run the Worker
CMD ["/app/worker"]
