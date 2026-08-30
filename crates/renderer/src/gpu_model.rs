use bytemuck::{Pod, Zeroable, bytes_of};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferUsages, Device, Queue, ShaderStages, TextureSampleType, TextureViewDimension, util::{BufferInitDescriptor, DeviceExt},
};

use crate::{GpuMesh, Model, texture::GpuTexture};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct MaterialUniform {
    base_color: [f32; 4],
}

pub struct GpuPrimitive {
    pub mesh: GpuMesh,
    pub material: usize,
}

pub struct GpuMaterial {
    pub bind_group: BindGroup,
    pub uniform_buffer: Buffer,
}

pub struct GpuModel {
    pub primitives: Vec<GpuPrimitive>,
    pub materials: Vec<GpuMaterial>,
    pub textures: Vec<GpuTexture>,
    pub fallback_texture: GpuTexture,
}

impl GpuModel {
    pub fn create_material_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("material bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }
    pub fn new(
        device: &Device,
        queue: &Queue,
        model: &Model,
        material_layout: &BindGroupLayout,
    ) -> Self {
        let textures = model
            .images
            .iter()
            .enumerate()
            .map(|(index, image)| {
                GpuTexture::from_image(device, queue, image, &format!("GLB texture {index}"))
            })
            .collect::<Vec<_>>();
        let fallback_texture = GpuTexture::white(device, queue);

        let materials = model.materials.iter().enumerate().map(|(index, material)| {
            let texture = material
                .base_color_texture
                .and_then(|texture_index| textures.get(texture_index))
                .unwrap_or(&fallback_texture);
            let uniform = MaterialUniform { base_color: material.base_color };
            let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("material uniform buffer"),
                contents: bytes_of(&uniform),
                usage: BufferUsages::UNIFORM
            });
            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some(&format!("material_bind_group {index}")),
                layout: material_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture.view)
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&texture.sampler),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: uniform_buffer.as_entire_binding()
                    }
                ]
            });
            GpuMaterial {
                bind_group,
                uniform_buffer
            }
        }).collect::<Vec<_>>();
        
        let primitives = model.meshes.iter().map(|mesh| {
            GpuPrimitive {
                mesh: GpuMesh::new(device, &mesh.vertices, &mesh.indices),
                material: mesh.material,
            }
        }).collect::<Vec<_>>();

        Self {
            primitives,
            materials,
            textures,
            fallback_texture
        }
    }
}
