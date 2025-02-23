# Stage 1: Build the Go application
FROM golang:1.24 AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy the Go modules manifest and download dependencies
COPY go.mod go.sum ./
RUN go mod download

# Copy the rest of the application
COPY . .

# Build the Go binary
RUN go build -o worker worker.go

# Stage 2: Create a runtime container
FROM ubuntu:22.04

# Set working directory
WORKDIR /app

# Install Docker inside the worker container
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    wget \
    gnupg \
    lsb-release

# Install Docker CLI (to support Dagger)
RUN curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt-get update && apt-get install -y docker-ce-cli

# Allow worker to use Docker socket (in Kubernetes, ensure you mount /var/run/docker.sock)
RUN ln -s /usr/bin/docker /usr/local/bin/docker

# Copy the compiled binary from the builder stage
COPY --from=builder /app/worker /app/worker
COPY client /client
COPY shared /shared
COPY python /python
# Expose the application port
EXPOSE 5001 5002

# Run the Worker
CMD ["/app/worker"]
