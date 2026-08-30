# Web model viewer

The web package renders the bundled breadboard GLB with WebGPU, falling back to
WebGL2 when WebGPU is unavailable. A user can choose another self-contained `.glb`
file (up to 100 MB) and render it without sending it to a server.

## Development

Install the `wasm32-unknown-unknown` Rust target and serve the web package with:

```bash
dx serve --package web --platform web
```
