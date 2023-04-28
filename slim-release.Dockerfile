FROM rust:1.67-bullseye AS builder
RUN apt-get update && \
    apt-get install -y \
    zip build-essential cmake pkg-config \
    libfontconfig1-dev clang libasound2-dev ninja-build \
    libxcb-xfixes0-dev mesa-vulkan-drivers
ADD . /build
WORKDIR /build
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && \
    apt-get install -y \
    libasound2
WORKDIR /app
COPY --from=builder /build/target/release/ambient ./
ADD guest/rust/examples /app/
CMD [ "./ambient", "serve", "examples/games/minigolf" ]
