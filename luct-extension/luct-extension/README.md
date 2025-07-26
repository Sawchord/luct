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

### Run

```
web-ext run --devtools
```