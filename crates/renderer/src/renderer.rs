use bytemuck::{Pod, Zeroable, bytes_of};
use glam::Mat4;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, DepthStencilState, Device, FragmentState, IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState, PushConstantRange, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, TextureFormat, TextureView, VertexState, util::{BufferInitDescriptor, DeviceExt},
};

use crate::{camera::Camera, depth::{DepthTexture}, mesh::{GpuMesh, Vertex}};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Globals {
    view_projection: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    light_direction: [f32; 4],
}

pub struct Renderer {
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    globals_buffer: Buffer,
    globals_bind_group: BindGroup,
    depth: DepthTexture,
    mesh: GpuMesh,
    camera: Camera,
    width: u32,
    height: u32,
    target_format: TextureFormat,
}

impl Renderer {
    pub fn new(
        device: &Device,
        queue: &Queue,
        target_format: TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let width = width.max(1);
        let height = height.max(1);

        let camera = Camera::new(width, height);

        let depth = DepthTexture::new(device, width, height);

        let mesh = GpuMesh::new(device, GpuMesh::CUBE_VERTICES, GpuMesh::CUBE_INDICES);

        let globals = Globals {
            view_projection: camera.view_projection_matrix().to_cols_array_2d(),
            model: Mat4::IDENTITY.to_cols_array_2d(),
            light_direction: [
                -0.4,
                -1.,
                -0.3,
                0.
            ],
        };

        let globals_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("3D renderer globals"),
            contents: bytes_of(&globals),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        },
        );

        let globals_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("3D renderer globals layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                }]
            }
        );

        let globals_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("3D renderer gobals bind group"),
            layout: &globals_layout,
            entries: &[BindGroupEntry { binding: 0, resource: globals_buffer.as_entire_binding() }]
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("3D mesh shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("3D mesh pipeline layout"),
            bind_group_layouts: &[&globals_layout],
            push_constant_ranges: &[]
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("3D mesh pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::LAYOUT],
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: DepthTexture::FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }), 
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[
                    Some(ColorTargetState {
                        format: target_format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })
                ]
            }),
            multiview: None,
            cache: None,
        });

        Self {
            device: device.clone(),
            queue: queue.clone(),
            pipeline,
            globals_buffer,
            globals_bind_group,
            depth,
            mesh,
            camera,
            width,
            height,
            target_format,
        }
    }

    pub fn resize(
        &mut self,
        width: u32, 
        height: u32,
    ) {
        let width = width.max(1);
        let height = height.max(1);
        
        if self.width == width && self.height == height {
            return
        }

        self.width = width;
        self.height = height;

        self.camera.resize(width, height);
        
        self.depth = DepthTexture::new(&self.device, width, height)
    }

    pub fn render(
        &mut self,
        target: &TextureView,
        width: u32,
        height: u32,
        time_seconds: f32,
    ) {
        self.resize(width, height);

        let model = Mat4::from_rotation_y(time_seconds* 0.7) * Mat4::from_rotation_x(time_seconds * 0.3);

        let globals = Globals {
            view_projection: self.camera.view_projection_matrix().to_cols_array_2d(),
            model: model.to_cols_array_2d(),
            light_direction: [-0.4, -1., -0.3, 0.]
        };

        self.queue.write_buffer(&self.globals_buffer, 0, bytes_of(&globals));

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {label: Some("3D render encoder")});
        
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("3D render pass"),
                color_attachments: &[
                    Some(RenderPassColorAttachment {
                        view: target,
                        depth_slice: None,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color { r: 0.02, g: 0.025, b: 0.035, a: 1.}),
                            store: StoreOp::Store,
                        },
                    })
                ],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(Operations { load: LoadOp::Clear(1.0), store: StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
    
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.globals_bind_group, &[]);
            pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint16);
            pass.draw_indexed(0..self.mesh.index_count, 0, 0..1);
        }
        
        self.queue.submit([encoder.finish()]);
    }
}