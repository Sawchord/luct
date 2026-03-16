# luCT extension

## Building

Assuming you have a recent `docker` installation, that supports `buildx`, 
building the extension requires a single line:

```
docker buildx build --progress=plain . -o .
```

You should find the `luct.zip` in the working directory

## Developing

### Setup

This setup assumes, you have rust, npm and firefox already installed on
your system.

In addition, the following tools are needed:

```
rustup target add wasm32-unknown-unknown
cargo install wasm-pack wasm-opt
npm install -g web-ext
```

### Build and run using wasm-pack

From `luct-extension/luct` run the following commands:

```
wasm-pack build ../luct-extension --out-dir ../luct/assets/wasm/ --target web --no-opt --no-typescript
web-ext run --devtools
```