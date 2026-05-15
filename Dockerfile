ARG BUILD_PLATFORM=linux/amd64
ARG RUST_VERSION=1.91
FROM --platform=${BUILD_PLATFORM} rust:${RUST_VERSION} AS builder

RUN rustup target add x86_64-unknown-linux-gnu

# Copy the entire workspace
COPY . /stylus-sdk-rs/
WORKDIR /stylus-sdk-rs

# Build cargo-stylus from the workspace
RUN cargo build --release -p cargo-stylus

FROM --platform=${BUILD_PLATFORM} rust:${RUST_VERSION} AS cargo-stylus-base
COPY --from=builder /stylus-sdk-rs/target/release/cargo-stylus /usr/local/bin/cargo-stylus
