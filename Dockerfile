ARG DEBIAN_FRONTEND=noninteractive

FROM debian:12-slim AS builder

RUN apt-get update -y && apt-get install -y \
  procps \
  net-tools \
  curl

# Install cargo
RUN curl https://sh.rustup.rs -sSf | \
  sh -s -- --profile minimal \
  --default-toolchain 1.82.0 \
  --component clippy \
  --component rustfmt \
  -y
ENV PATH="$PATH:/root/.cargo/bin"

WORKDIR /app

FROM builder AS local
