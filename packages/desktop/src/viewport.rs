use std::time::Instant;

use dioxus_native::{CustomPaintCtx, CustomPaintSource, DeviceHandle, TextureHandle};
use renderer::{Model, Renderer};
use wgpu::{Device, Extent3d, Queue, Texture, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, naga::compact::KeepUnused::No, wgt::TextureDescriptor};

const TARGET_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

pub struct Viewport {
    model: Model,
    renderer: Option<Renderer>,
    device: Option<Device>,
    queue: Option<Queue>,
    texture: Option<Texture>,
    texture_handle: Option<TextureHandle>,
    width: u32,
    height: u32,
    start: Instant,
}

impl Viewport {
    pub fn new(model_bytes: &[u8]) -> Self {
        let model = Model::from_glb_bytes(model_bytes).expect("Failed to load a model.");

        println!(
            "GLB loaded: {} primitives, {} materials, {} images.",
            model.meshes.len(),
            model.materials.len(),
            model.images.len()
        );
        
        Self {
            model,
            renderer: None,
            device: None,
            queue: None,
            texture: None, 
            texture_handle: None,
            width: 0,
            height: 0, 
            start: Instant::now(),
        }
    }

    fn recreate_texture(
        &mut self,
        ctx: &mut CustomPaintCtx<'_>,
        width: u32,
        height: u32,
    ) {
        if let Some(handle) = self.texture_handle.take() {
            ctx.unregister_texture(handle);
        }

        let device = self.device.as_ref().expect("WGPU device unavailable");

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("3D viewport"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[]
        });
        
        let handle = ctx.register_texture(texture.clone());

        self.texture = Some(texture);
        self.texture_handle = Some(handle);
        self.width = width;
        self.height = height;
    }
}

impl CustomPaintSource for Viewport {
    fn resume(&mut self, gpu: &DeviceHandle) {
        self.device = Some(gpu.device.clone());
        self.queue = Some(gpu.queue.clone());
        
        self.renderer = Some(Renderer::new(&gpu.device, &gpu.queue, TARGET_FORMAT, 1, 1))
    }
    fn suspend(&mut self) {
        self.device = None;
        self.texture = None;
        self.texture_handle = None;
        self.renderer = None;

        self.width = 0;
        self.height = 0;
    }
    fn render(
        &mut self,
        mut ctx: CustomPaintCtx<'_>,
        width: u32,
        height: u32,
        scale: f64,
    ) -> Option<TextureHandle>
    {
        if width == 0 || height == 0 {
            return None
        }
        if self.texture.is_none() || self.width != width || self.height != height {
            self.recreate_texture(&mut ctx, width, height);
        }

        let texture = self.texture.as_ref()?;
        let view = texture.create_view(&TextureViewDescriptor::default());
        let renderer = self.renderer.as_mut()?;
        renderer.render(&view, width, height, self.start.elapsed().as_secs_f32());
        
        self.texture_handle.clone()
    }
}
