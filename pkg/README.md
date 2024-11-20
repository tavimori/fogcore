# Fogcore

Fogcore is a library for rendering tracks data from the Fog of World App. 

The library is built with Rust and WebGPU, and can be used in WebAssembly or natively.

## Development

### 📦 Build the WASM package

```
wasm-pack build --target web --features wasm --no-default-features
```

### Build the native library

```
cargo build
```

