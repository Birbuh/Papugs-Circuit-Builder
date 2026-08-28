use wgpu::{
    Device, Extent3d, Texture, TextureDescriptor, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

pub struct DepthTexture {
    _texture: Texture,
    pub view: TextureView,
}

impl DepthTexture {
    pub const FORMAT: TextureFormat = { TextureFormat::Depth32Float };

    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("depth texture"),
            size: Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default(),);
        Self {
            _texture: texture,
            view,
        }
    }
}
