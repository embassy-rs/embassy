# WASM example

Examples use a CLI tool named `wasm-pack` to build this example:

```
cargo install wasm-pack --version 0.12.1
```

## Building

To build the example, run:

```
wasm-pack build --target web
```

## Running

To run the example, start a webserver server the local folder:


```
python -m http.server
```

Then, open a browser at http://127.0.0.1:8000 and watch the ticker print entries to the window.
