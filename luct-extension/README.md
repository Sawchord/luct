# luCT extension

## Tooling

### Setup

This setup assumes, you have rust, npm and firefox already installed on
your system.

In addition, the following tools are needed:

```
rustup target add wasm32-none-none
cargo install wasm-pack wasm-opt
npm install -g web-ext
```

### Build and run using wasm-pack

From `luct-extension/luct` run the following commands:

```
wasm-pack build ../luct-extension --out-dir ../luct/assets/wasm/ --release --target web
web-ext run --devtools
```

### Building without wasm-pack

**Note**: For this approach, you need to install `wasm-bindgen-cli` manually using

```
cargo install -f wasm-bindgen-cli --version <version>
```

Also pay close attention that `<version>` matches the version of your `Cargo.lock` file.

Build the wasm package

```
cargo build -p luct-extension --release --target wasm32-unknown-unknown
```

Generate the bindgen
```
wasm-bindgen target/wasm32-unknown-unknown/release/luct_extension.wasm --target web --out-dir luct-extension/luct/assets/wasm
```
