#[cfg(target_arch = "wasm32")]
mod viewport;

#[cfg(target_arch = "wasm32")]
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use dioxus_web::WebEventExt;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

#[cfg(target_arch = "wasm32")]
use viewport::WebViewport;

#[cfg(target_arch = "wasm32")]
const FAVICON: Asset = asset!("/assets/favicon.ico");
#[cfg(target_arch = "wasm32")]
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct UploadRequest {
    name: String,
    bytes: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct ViewportStatus {
    message: String,
    is_error: bool,
}

#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(App);
}

// `cargo check -p web` runs for the host by default. The actual application is
// compiled for wasm32 by `dx serve --platform web`.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("The web app must be built for wasm32-unknown-unknown.");
}

#[cfg(test)]
mod tests {
    use renderer::Model;

    #[test]
    fn bundled_model_is_a_renderable_glb() {
        let model = Model::from_glb_bytes(include_bytes!("../assets/breadboard.glb"))
            .expect(":(((((((((");

        assert!(!model.meshes.is_empty());
        assert!(model.meshes.iter().all(|mesh| !mesh.vertices.is_empty()));
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
fn App() -> Element {
    rsx! {
        document::Title { "Papug's Ducking Render Engine" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        ModelViewport {}
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
fn ModelViewport() -> Element {
    let pending_upload = use_signal(|| None::<UploadRequest>);
    let status = use_signal(|| ViewportStatus {
        message: "Starting the 3D renderer…".to_string(),
        is_error: false,
    });

    let status_value = status.read().clone();
    let status_class = if status_value.is_error {
        "viewer-status viewer-status--error"
    } else {
        "viewer-status"
    };

    let mut upload_for_input = pending_upload;
    let mut status_for_input = status;
    let mut upload_for_canvas = pending_upload;
    let mut status_for_canvas = status;

    rsx! {
        main { class: "viewer-shell",
            canvas {
                class: "viewer-canvas",
                aria_label: "3D model preview",
                onmounted: move |event| {
                    let element = event.as_web_event();
                    let canvas = element
                        .dyn_into::<HtmlCanvasElement>()
                        .expect("mounted element isn't a canvas");

                    spawn(async move {
                        let mut viewport = match WebViewport::new(canvas).await {
                            Ok(viewport) => viewport,
                            Err(error) => {
                                status_for_canvas.set(ViewportStatus {
                                    message: error,
                                    is_error: true,
                                });
                                return;
                            }
                        };

                        status_for_canvas.set(ViewportStatus {
                            message: "Rendering the sample breadboard".to_string(),
                            is_error: false,
                        });
                        let start = js_sys::Date::now();

                        loop {
                            if upload_for_canvas.peek().is_some() {
                                if let Some(upload) = upload_for_canvas.write().take() {
                                    match viewport.load_model(&upload.bytes) {
                                        Ok(summary) => status_for_canvas.set(ViewportStatus {
                                            message: format!(
                                                "Rendering {} · {} primitives · {} materials",
                                                upload.name, summary.primitives, summary.materials,
                                            ),
                                            is_error: false,
                                        }),
                                        Err(error) => status_for_canvas.set(ViewportStatus {
                                            message: format!("Could not render {}: {error}", upload.name),
                                            is_error: true,
                                        }),
                                    }
                                }
                            }

                            let time = ((js_sys::Date::now() - start) / 1000.0) as f32;
                            viewport.render(time);
                            gloo_timers::future::TimeoutFuture::new(16).await;
                        }
                    });
                },
            }

            section { class: "upload-panel",
                div { class: "upload-panel__copy",
                    p { class: "eyebrow", "Papug's Ducking Render Engine!" }
                    h1 { "GLB viewer" }
                    p { class: "upload-hint", "Choose a self-contained GLB file to render it locally in your browser." }
                }

                label { class: "upload-button", r#for: "model-upload",
                    span { "Choose GLB" }
                    input {
                        id: "model-upload",
                        r#type: "file",
                        accept: ".glb,model/gltf-binary",
                        onchange: move |event| {
                            let Some(file) = event.files().into_iter().next() else {
                                return;
                            };

                            let name = file.name();
                            if !name.to_ascii_lowercase().ends_with(".glb") {
                                status_for_input.set(ViewportStatus {
                                    message: "Please choose a .glb file.".to_string(),
                                    is_error: true,
                                });
                                return;
                            }

                            status_for_input.set(ViewportStatus {
                                message: format!("Loading {name}…"),
                                is_error: false,
                            });
                            spawn(async move {
                                match file.read_bytes().await {
                                    Ok(bytes) => upload_for_input.set(Some(UploadRequest {
                                        name,
                                        bytes: bytes.to_vec(),
                                    })),
                                    Err(error) => status_for_input.set(ViewportStatus {
                                        message: format!("Could not read the selected file: {error}"),
                                        is_error: true,
                                    }),
                                }
                            });
                        },
                    }
                }

                p {
                    class: status_class,
                    role: "status",
                    aria_live: "polite",
                    "{status_value.message}"
                }
            }
        }
    }
}
