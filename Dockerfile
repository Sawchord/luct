# Compile the was
FROM rust:1.92-alpine AS wasm-builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install -f wasm-bindgen-cli --version 0.2.108

WORKDIR /usr/src

COPY . .
RUN mkdir target
RUN cargo build -p luct-extension --release --target wasm32-unknown-unknown
RUN wasm-bindgen target/wasm32-unknown-unknown/release/luct_extension.wasm --target web --out-dir target/wasm

# Build the extension
FROM alpine:3.23.3 AS extension-packager

RUN apk add zip tree

WORKDIR /usr/src

COPY ./luct-extension/luct luct
COPY --from=wasm-builder /usr/src/target/wasm luct/assets

RUN tree
RUN zip -r -FS /luct.zip luct
RUN sha256sum /luct.zip
