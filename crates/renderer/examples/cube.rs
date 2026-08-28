// THIS FILE IS AI GENERATED FOR TESTING PURPOSES.


use std::sync::Arc;

use renderer::Renderer;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{
        ActiveEventLoop,
        EventLoop,
    },
    window::{
        Window,
        WindowId,
    },
};

struct State {
    window: Arc<Window>,

    surface: wgpu::Surface<'static>,
    device: wgpu::Device,

    config: wgpu::SurfaceConfiguration,

    renderer: Renderer,

    start: std::time::Instant,
}

impl State {
    async fn new(
        window: Arc<Window>,
    ) -> Self {
        let size =
            window.inner_size();

        /*
         * WGPU INSTANCE
         */

        let instance =
            wgpu::Instance::default();

        /*
         * NATIVE WINDOW SURFACE
         */

        let surface =
            instance
                .create_surface(
                    window.clone(),
                )
                .expect(
                    "failed to create surface"
                );

        /*
         * GPU
         */

        let adapter =
            instance
                .request_adapter(
                    &wgpu::RequestAdapterOptions {
                        power_preference:
                            wgpu::PowerPreference::
                                HighPerformance,

                        compatible_surface:
                            Some(&surface),

                        force_fallback_adapter:
                            false,
                    },
                )
                .await
                .expect(
                    "failed to find GPU adapter"
                );

        println!(
            "GPU: {:?}",
            adapter.get_info()
        );

        let (device, queue) =
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor::default(),
                )
                .await
                .expect(
                    "failed to create device"
                );

        /*
         * SURFACE CONFIG
         */

        let config =
            surface
                .get_default_config(
                    &adapter,
                    size.width.max(1),
                    size.height.max(1),
                )
                .expect(
                    "surface is unsupported"
                );

        surface.configure(
            &device,
            &config,
        );

        /*
         * OUR 3D RENDERER
         */

        let renderer =
            Renderer::new(
                &device,
                &queue,
                config.format,
                config.width,
                config.height,
            );

        Self {
            window,

            surface,
            device,

            config,

            renderer,

            start:
                std::time::Instant::now(),
        }
    }

    fn resize(
        &mut self,
        size: PhysicalSize<u32>,
    ) {
        if
            size.width == 0
            || size.height == 0
        {
            return;
        }

        self.config.width =
            size.width;

        self.config.height =
            size.height;

        self.surface.configure(
            &self.device,
            &self.config,
        );

        self.renderer.resize(
            size.width,
            size.height,
        );
    }

    fn render(
        &mut self,
    ) {
        /*
         * GET CURRENT WINDOW FRAME
         */

        let frame =
            match
                self.surface
                    .get_current_texture()
            {
                Ok(frame) => frame,

                Err(
                    wgpu::SurfaceError::Lost
                    | wgpu::SurfaceError::Outdated,
                ) => {
                    self.surface.configure(
                        &self.device,
                        &self.config,
                    );

                    return;
                }

                Err(
                    wgpu::SurfaceError::Timeout
                ) => {
                    return;
                }

                Err(
                    wgpu::SurfaceError::OutOfMemory
                ) => {
                    panic!(
                        "GPU out of memory"
                    );
                }

                Err(error) => {
                    eprintln!(
                        "surface error: {error:?}"
                    );

                    return;
                }
            };

        /*
         * TEXTUREVIEW THAT OUR RENDERER NEEDS
         */

        let view =
            frame
                .texture
                .create_view(
                    &wgpu::TextureViewDescriptor::default(),
                );

        /*
         * THIS IS THE IMPORTANT PART.
         *
         * We give our 3D renderer a TextureView.
         */

        self.renderer.render(
            &view,

            self.config.width,
            self.config.height,

            self.start
                .elapsed()
                .as_secs_f32(),
        );

        /*
         * DISPLAY THE COMPLETED FRAME
         */

        frame.present();
    }
}


#[derive(Default)]
struct App {
    state: Option<State>,
}


impl ApplicationHandler for App {
    fn resumed(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) {
        if self.state.is_some() {
            return;
        }

        let window =
            Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title(
                                "Renderer Test"
                            )
                            .with_inner_size(
                                PhysicalSize::new(
                                    1280,
                                    720,
                                )
                            ),
                    )
                    .expect(
                        "failed to create window"
                    ),
            );

        let state =
            pollster::block_on(
                State::new(
                    window,
                )
            );

        state
            .window
            .request_redraw();

        self.state =
            Some(state);
    }


    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) =
            self.state.as_mut()
        else {
            return;
        };

        if
            window_id
            != state.window.id()
        {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(
                size
            ) => {
                state.resize(
                    size
                );
            }

            WindowEvent::RedrawRequested => {
                state.render();

                /*
                 * Schedule another frame.
                 */

                state
                    .window
                    .request_redraw();
            }

            _ => {}
        }
    }
}


fn main() {
    let event_loop =
        EventLoop::new()
            .expect(
                "failed to create event loop"
            );

    let mut app =
        App::default();

    event_loop
        .run_app(
            &mut app
        )
        .expect(
            "event loop failed"
        );
}