FROM rustlang/rust:nightly-slim as builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y build-essential git clang curl libssl-dev llvm libudev-dev make protobuf-compiler pkg-config && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown
RUN cargo build --release

FROM debian:buster-slim
EXPOSE 30333 9933 9944
VOLUME /data
COPY --from=builder /app/target/release/substrate /node
COPY specs /specs
RUN useradd -Ms /bin/bash liberland
RUN chown liberland:liberland -R /specs /node /data
USER liberland
ENTRYPOINT [ "/node" ]
