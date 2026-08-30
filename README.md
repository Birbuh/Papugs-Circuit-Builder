# Papugs-Ducking-Render-Engine

Papugs-Ducking-Render-Engine is an experimental 3D rendering engine written in
Rust with Dioxus and wgpu. It explores a shared rendering stack for web and
native applications, with GLB model loading and GPU-backed viewports.

> [!NOTE]
> The engine is an early prototype. The browser renderer is currently the most complete frontend; the desktop and mobile targets are still experimental. They are NOT going to be updated by me (birbuh) in the future. Reason: I don't care and I'm sick of this project. 

## What works today

The web application can:

- render the bundled breadboard model;
- open a self-contained `.glb` file from the user's device;
- display triangle meshes, base-color materials, and embedded textures;
- automatically center and scale imported geometry;
- resize the canvas for the window and the display's pixel density;
- use WebGPU when available and fall back to WebGL2 when necessary; and
- report unsupported or invalid models without replacing the last valid scene.

Selected files are processed entirely in the browser and are not sent to a
server. Only binary, self-contained GLB files are accepted; external `.gltf` resources are not supported by the viewer.

## Running the web app

The app is currently hosted on https://birbuh.github.io/Papugs-Ducking-Render-Engine/

If you wish to host it yourself:

### Prerequisites

- A recent Rust toolchain
- The `wasm32-unknown-unknown` Rust target
- Dioxus CLI 0.7
- A browser with WebGPU or WebGL2 support

Install the Rust target and Dioxus CLI if needed:

```bash
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version 0.7.10 --locked
```

From the repository root, start the development server:

```bash
dx serve --package web --platform web
```

Open the local URL printed by Dioxus. The bundled breadboard should appear in
the viewport; use **Choose GLB** to render another model.

## Checks

Check the host-side package setup:

```bash
cargo check -p web
```

Check the browser build:

```bash
cargo check -p web --target wasm32-unknown-unknown --features web
```

Run the regression test for the bundled model:

```bash
cargo test -p web --bin web
```

## Repository layout

```text
Papugs-Ducking-Render-Engine/
├── crates/
│   └── renderer/       Shared camera, mesh, GLB parsing, and wgpu code
├── packages/
│   ├── web/            Active browser GLB viewer
│   ├── desktop/        Experimental native Dioxus renderer
│   ├── mobile/         Mobile application scaffold
│   ├── ui/             Shared Dioxus UI components
│   └── api/            Full-stack server-function scaffold
├── patches/                Local patches used by the native renderer
├── docs/                   Generated static web output
├── Cargo.toml              Rust workspace configuration
└── Dioxus.toml             Dioxus application configuration
```

The desktop and mobile packages are prototypes and do not yet provide the same
model-loading experience as the web application.

## Model credit

The bundled model is
[Double-Sided Perfboard 7×5 cm – 3D Model](https://skfb.ly/pFBPx) by Alex human,
used under the
[Creative Commons Attribution 4.0 license](https://creativecommons.org/licenses/by/4.0/).
