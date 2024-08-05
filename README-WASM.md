To build the wasm target (and package):

```
wasm-pack build --target web --features wasm  
```

The wasm target is more for development use (real-time subjective evaluation of the rendering process).

The js package of fogcore provides reduced functionality: currently only supports loading and rendering of the fow data.