ARG DEBIAN_FRONTEND=noninteractive

FROM debian:12-slim AS builder

RUN apt-get update -y && apt-get install -y \
  procps \
  net-tools \
  vim \
  curl \
  build-essential \
  libssl-dev \
  pkg-config \
  protobuf-compiler

WORKDIR /app

# Install cargo
RUN curl https://sh.rustup.rs -sSf | \
  sh -s -- --profile minimal \
  --default-toolchain 1.84.0 \
  --component clippy \
  --component rustfmt \
  -y
ENV PATH="$PATH:/root/.cargo/bin"
