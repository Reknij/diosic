# Rust
FROM rust:latest AS builder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
WORKDIR /myapp
COPY ./server/ .
RUN cargo build --target x86_64-unknown-linux-musl --release

# Node
FROM node:16-alpine as node_build
WORKDIR /myapp
COPY ./web/ .
RUN npm install
RUN npm run build

# Alpine
FROM alpine:3.17
COPY --from=builder /myapp/target/x86_64-unknown-linux-musl/release/diosic /
COPY --from=node_build /myapp/dist/ /webpage
VOLUME ["/library", "/data"]
ENV LIB_NAME_1="My Library"
CMD /diosic -d /data -l "${LIB_NAME_1};/library"