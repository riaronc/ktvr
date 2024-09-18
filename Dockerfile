# Dockerfile

# Stage 1: Build
FROM rust:1.81 as builder

# Set environment variables
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=$CARGO_HOME/bin:$PATH

# Create app directory
WORKDIR /app

# Copy Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock ./

# Fetch dependencies
RUN cargo fetch

# Copy source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Stage 2: Runtime
FROM ubuntu:22.04

# Install required libraries
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user and group
RUN groupadd -r appgroup && useradd -r -g appgroup appuser

# Set working directory
WORKDIR /home/appuser

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/url_shortener .

# Change ownership to the non-root user
RUN chown appuser:appgroup url_shortener

# Switch to the non-root user
USER appuser

# Expose container port
EXPOSE ${CONTAINER_PORT}

# Set environment variables
ENV RUST_LOG=${RUST_LOG}

# Run the application
CMD ["./url_shortener"]
