# Build stage - compile Rust binary with newer Rust
FROM rustlang/rust:nightly AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY prompts ./prompts
RUN cargo build --release

# Runtime stage - use HyperFrames renderer as base
FROM hyperframes-renderer:0.8.143 AS runtime

USER root

# Copy the compiled binary
COPY --from=builder /app/target/release/anim-cli /usr/local/bin/anim-cli

# Create non-root user for security
RUN useradd -m -s /bin/bash appuser && \
    chown -R appuser:appuser /home/appuser

USER appuser
WORKDIR /home/appuser

# Allow running arbitrary commands, default to anim-cli
ENTRYPOINT ["anim-cli"]