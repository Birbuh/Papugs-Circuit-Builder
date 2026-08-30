use std::time::Duration;

use dioxus::prelude::*;

use dioxus_native::use_wgpu;
use ui::Navbar;
use views::{Blog, Home};

use crate::viewport::Viewport;

mod views;
mod viewport;
mod render;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(DesktopNavbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        // Global app resources
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        // Router::<Route> {}

        ModelViewport {}
    }
}

/// A desktop-specific Router around the shared `Navbar` component
/// which allows us to use the desktop-specific `Route` enum.
#[component]
fn DesktopNavbar() -> Element {
    rsx! {
        Navbar {
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
        }

        Outlet::<Route> {}
    }
}

#[component]
fn ModelViewport() -> Element {
    let mut frame = use_signal(|| 0u64);

    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_millis(17)).await;
            frame += 1
        }
    });
    
    let paint_id = use_wgpu(|| Viewport::new(include_bytes!("../assets/breadboard.glb")));

    rsx! {
        canvas { 
            "src": "{paint_id}",

            style: "
                display: block;
                flex: 1;
                width: 600;
                height: 400;
                min-width: 600;
                min-height: 400;
            ",
        }
    }
}