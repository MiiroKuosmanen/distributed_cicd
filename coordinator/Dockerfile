# Use Rust as the base image
FROM rust:1.84 AS builder

# Set working directory
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates

# Copy the Rust project files
COPY . .

# Build the coordinator binary
RUN cargo build --release

# Create a minimal runtime image
FROM ubuntu:22.04

# Set working directory
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/coordinator /app/coordinator

# Expose the application port
EXPOSE 3000

# Run the Coordinator
CMD ["/app/coordinator"]
