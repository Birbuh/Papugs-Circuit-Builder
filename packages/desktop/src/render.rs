use std::time::Instant;

use renderer::Renderer;
use wgpu::{Device, Extent3d, Texture, TextureDimension, TextureFormat, TextureUsages, wgt::{TextureDescriptor, TextureViewDescriptor}};
use dioxus_native::{CustomPaintSource, DeviceHandle, TextureHandle};

pub struct Native3dViewport {
    renderer: Option<Renderer>,
    device: Option<Device>,
    texture: Option<Texture>,
    texture_handle: Option<TextureHandle>,
    width: u32,
    height: u32,
    start: Instant,
}

impl Native3dViewport {
    pub fn new() -> Self {
        Self {
            renderer: None,
            device: None,
            texture: None,
            texture_handle: None,
            width: 0,
            height: 0,
            start: Instant::now(),
        }
    }
}

impl CustomPaintSource for Native3dViewport {
    fn resume(&mut self, gpu: &DeviceHandle) {
        self.device = Some(gpu.device.clone());
        self.renderer = Some(Renderer::new(&gpu.device, &gpu.queue, TextureFormat::Rgba8Unorm, 1, 1));
    }

    fn suspend(&mut self) {
        self.renderer = None;
        self.texture = None;
        self.texture_handle = None;
        self.device = None
    }

    fn render(
        &mut self,
        mut ctx: dioxus_native::CustomPaintCtx<'_>,
        width: u32,
        height: u32,
        scale: f64,
    ) -> Option<TextureHandle>
    {
        if width == 0 || height == 0 {
            return None
        }

        if self.texture.is_none() || self.width != width || self.height != height {
            if let Some(old) = self.texture_handle.take() {
                ctx.unregister_texture(old);
            }
        }

        let device = self.device.as_ref()?;
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("3D viewport"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[]
        });

        self.texture_handle = Some(ctx.register_texture(texture.clone()));
        self.texture = Some(texture);
        self.width = width;
        self.height = height;

        let texture = self.texture.as_ref()?;
        let view = texture.create_view(&TextureViewDescriptor::default());
        let renderer = self.renderer.as_mut()?;

        renderer.render(&view, width, height, self.start.elapsed().as_secs_f32());
        
        self.texture_handle.clone()
    }
}