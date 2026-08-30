use wgpu::{AddressMode, Device, Extent3d, FilterMode, Origin3d, Queue, Sampler, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureFormat, TextureUsages, TextureView, wgt::{SamplerDescriptor, TextureViewDescriptor}};

use crate::Image;

pub struct GpuTexture {
    pub texture: Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl GpuTexture {
    pub fn from_image(
        device: &Device,
        queue: &Queue,
        image: &Image,
        label: &str,
    ) -> Self {
        Self::from_rgba8(device, queue, image.width, image.height, &image.pixels, label)
    }

    pub fn white(device: &Device, queue: &Queue) -> Self {
        Self::from_rgba8(device, queue, 1, 1, &[ 255, 255, 255, 255 ], "white fallback texture")
    }

    pub fn from_rgba8(
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        pixels: &[u8],
        label: &str
    ) -> Self {
        let width = width.max(1);
        let height = height.max(1);

        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[]
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All
            }, 
            pixels, 
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height)
            },
            size
        );

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("material texture sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Self { texture, view, sampler }
    }
}