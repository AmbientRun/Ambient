# Instructions (replace "guest/rust/examples/games/minigolf" with your project):
# 1. Build the image: docker build -f slim-release.Dockerfile -t ambient .
# 2. Run the image: docker run -p 8999:8999/tcp -p 9000:9000/udp -v `pwd`/guest/rust/examples/games/minigolf:/app/project ambient
#    Note: the project has to be built (`ambient build guest/rust/examples/games/minigolf`)
# 3. Run `ambient join` locally to connect to the server

FROM rust:1.70-bullseye AS builder
RUN apt-get update && \
    apt-get install -y \
    zip build-essential cmake pkg-config \
    libfontconfig1-dev clang libasound2-dev ninja-build \
    libxcb-xfixes0-dev mesa-vulkan-drivers
ADD . /build
WORKDIR /build
RUN cargo build --release --no-default-features --features slim
RUN strip target/release/ambient

FROM debian:bullseye-slim
RUN apt-get update && \
    apt-get install -y \
    libasound2
WORKDIR /app
COPY --from=builder /build/target/release/ambient ./
CMD [ "./ambient", "serve", "--public-host", "localhost", "--no-build", "project" ]
