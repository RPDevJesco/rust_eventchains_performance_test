# Multi-stage Dockerfile for EventChains Performance Benchmarking
# Includes perf, valgrind, and other profiling tools

FROM rust:1.83-slim AS builder

# Install build dependencies and profiling tools
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    linux-perf \
    valgrind \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Copy Rust project files
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

ENV RUSTFLAGS='-C target-cpu=native'

# Build the Rust benchmarks in release mode
RUN cargo build --release

# Runtime stage with profiling tools
FROM debian:bookworm-slim

# Install runtime dependencies and profiling tools
RUN apt-get update && apt-get install -y \
    linux-perf \
    valgrind \
    strace \
    time \
    htop \
    sysstat \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for running benchmarks
RUN useradd -m -s /bin/bash benchuser

WORKDIR /benchmark

# Copy built binaries from builder
COPY --from=builder /workspace/target/release/dijkstra_eventchains ./

# Copy source files for reference
COPY --from=builder /workspace/src/ ./src/

# Set appropriate permissions
RUN chown -R benchuser:benchuser /benchmark

# Create output directory for results
RUN mkdir -p /benchmark/results && chown benchuser:benchuser /benchmark/results

# Switch to non-root user
USER benchuser

# Set performance-related environment variables
ENV RUST_BACKTRACE=1
ENV RUST_LOG=info

# Default command runs the benchmark
CMD ["./dijkstra_eventchains"]

# --- Usage Instructions ---
# Build: docker build -t eventchains-bench .
# Run basic benchmark: docker run --rm eventchains-bench
# Run with perf (requires privileged mode):
#   docker run --rm --privileged --cap-add=SYS_ADMIN eventchains-bench \
#     perf stat -d ./dijkstra_eventchains
# Run with valgrind:
#   docker run --rm eventchains-bench valgrind --tool=massif ./dijkstra_eventchains
# Interactive shell:
#   docker run --rm -it eventchains-bench /bin/bash
