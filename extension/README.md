# luCT extension

## Building

Assuming you have a recent `docker` installation, that supports `buildx`, 
building the extension requires a single line:

```
docker buildx build --progress=plain . -o .
```

You should find the `luct.xpi` in the working directory

## Developing

### Setup wasm side

This setup assumes, you have rust, npm and firefox already installed on
your system.

In addition, the following tools are needed:

```
rustup target add wasm32-unknown-unknown
cargo install wasm-pack wasm-opt
```

### Build and run using wasm-pack

From `extension/luct` run the following commands:

```
wasm-pack build ../../luct-extension --out-dir ../extension/luct/wasm/ --target web --no-opt --no-typescript
```

### Javascript side

Inside of luct, run:

```
npm install
npm run dev
```

### Execute debugging firefox

```
npm install -g web-ext
web-ext run --devtools
```