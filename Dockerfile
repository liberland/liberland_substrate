FROM debian:buster-slim as rusty
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y build-essential git clang curl libssl-dev llvm libudev-dev make protobuf-compiler pkg-config && rm -rf /var/lib/apt/lists/*
RUN curl https://sh.rustup.rs -sSf | CARGO_HOME=/usr/local RUSTUP_HOME=/usr/local sh -s -- --default-toolchain nightly -y
RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
WORKDIR /app

FROM rusty as builder
COPY . .
RUN cargo build --release

FROM debian:buster-slim AS runtime
EXPOSE 30333 9933 9944
VOLUME /data
COPY --from=builder /app/target/release/substrate /node
COPY specs /specs
RUN useradd -Ms /bin/bash liberland
RUN mkdir /data && chown liberland:liberland -R /specs /node /data
USER liberland
ENTRYPOINT [ "/node" ]
