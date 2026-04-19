# Compile the wasm
FROM rust:1.95.0-alpine3.22 AS wasm-builder

RUN rustup target add wasm32-unknown-unknown
WORKDIR /usr/src

COPY . .
RUN mkdir target
RUN cargo build -p luct-extension --release --locked --target wasm32-unknown-unknown

# Build the extension
FROM python:3.14.4-alpine3.22 AS extension-packager

RUN apk add tree
WORKDIR /usr/src

RUN wget -c https://github.com/wasm-bindgen/wasm-bindgen/releases/download/0.2.117/wasm-bindgen-0.2.117-x86_64-unknown-linux-musl.tar.gz -O - | tar -xz

COPY ./extension/luct ./luct
COPY ./extension/zip.py .
COPY --from=wasm-builder /usr/src/target/wasm32-unknown-unknown/release/luct_extension.wasm .
RUN sha256sum luct_extension.wasm

RUN ./wasm-bindgen-0.2.117-x86_64-unknown-linux-musl/wasm-bindgen luct_extension.wasm --no-typescript --target web --out-dir luct/assets/wasm
RUN tree luct

RUN ./zip.py
RUN sha256sum luct.xpi

# Copy extension into empty exporter
FROM scratch AS exporter
COPY --from=extension-packager /usr/src/luct.xpi .
