use bytemuck::{bytes_of, Pod, Zeroable};
use glam::{Mat4, Vec3};
use renderer::{Camera, GpuMesh, Model, Vertex};
use web_sys::HtmlCanvasElement;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, DepthStencilState, Device, Extent3d, FilterMode, FragmentState,
    IndexFormat, LoadOp, MultisampleState, Operations, Origin3d, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, Sampler, SamplerBindingType,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration,
    SurfaceError, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDescriptor, TextureViewDimension, VertexState,
};

const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

pub struct ModelSummary {
    pub primitives: usize,
    pub materials: usize,
}

pub struct WebViewport {
    canvas: HtmlCanvasElement,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    renderer: ModelRenderer,
}

impl WebViewport {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        let (width, height) = canvas_size(&canvas);
        canvas.set_width(width);
        canvas.set_height(height);

        // `navigator.gpu` can exist even when the browser cannot actually
        // provide a WebGPU adapter (notably on some Linux configurations).
        // This performs a real support check and selects WebGL2 when needed.
        let instance =
            wgpu::util::new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor::default())
                .await;
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|error| format!("Could not create the 3D canvas: {error}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|error| {
                format!("Neither WebGPU nor WebGL2 is available in this browser: {error}")
            })?;
        let adapter_limits = adapter.limits();
        let required_limits = wgpu::Limits::downlevel_webgl2_defaults()
            .using_resolution(adapter_limits.clone())
            .using_alignment(adapter_limits);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("web model renderer"),
                required_limits,
                ..Default::default()
            })
            .await
            .map_err(|error| format!("Could not create a graphics device: {error}"))?;
        let config = surface
            .get_default_config(&adapter, width, height)
            .ok_or_else(|| "The browser does not support this canvas format.".to_string())?;
        surface.configure(&device, &config);

        let initial_model = Model::from_glb_bytes(include_bytes!("../assets/breadboard.glb"))
            .map_err(|error| format!("Could not load the sample model: {error:?}"))?;
        let renderer = ModelRenderer::new(
            &device,
            &queue,
            config.format,
            width,
            height,
            &initial_model,
        )?;

        Ok(Self {
            canvas,
            surface,
            device,
            queue,
            config,
            renderer,
        })
    }

    pub fn load_model(&mut self, bytes: &[u8]) -> Result<ModelSummary, String> {
        let model = Model::from_glb_bytes(bytes)
            .map_err(|error| format!("invalid or unsupported GLB ({error:?})"))?;
        let summary = ModelSummary {
            primitives: model.meshes.len(),
            materials: model.materials.len().saturating_sub(1),
        };
        let scene = Scene::new(
            &self.device,
            &self.queue,
            &model,
            &self.renderer.material_layout,
        )?;
        self.renderer.scene = scene;
        Ok(summary)
    }

    pub fn render(&mut self, time_seconds: f32) {
        let (width, height) = canvas_size(&self.canvas);
        if width != self.config.width || height != self.config.height {
            self.resize(width, height);
        }

        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(SurfaceError::Timeout) => return,
            Err(SurfaceError::OutOfMemory) => {
                web_sys::console::error_1(&"WebGPU ran out of memory.".into());
                return;
            }
            Err(error) => {
                web_sys::console::error_1(&format!("Surface error: {error:?}").into());
                return;
            }
        };

        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        self.renderer
            .render(&view, self.config.width, self.config.height, time_seconds);
        frame.present();
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.renderer.resize(width, height);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Globals {
    view_projection: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    light_direction: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct MaterialUniform {
    base_color: [f32; 4],
}

struct ModelRenderer {
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    globals_buffer: Buffer,
    globals_bind_group: BindGroup,
    material_layout: BindGroupLayout,
    depth: DepthTexture,
    camera: Camera,
    scene: Scene,
}

impl ModelRenderer {
    fn new(
        device: &Device,
        queue: &Queue,
        target_format: TextureFormat,
        width: u32,
        height: u32,
        model: &Model,
    ) -> Result<Self, String> {
        let camera = Camera::new(width, height);
        let depth = DepthTexture::new(device, width, height);
        let globals = Globals {
            view_projection: camera.view_projection_matrix().to_cols_array_2d(),
            model: Mat4::IDENTITY.to_cols_array_2d(),
            light_direction: [-0.4, -1.0, -0.3, 0.0],
        };
        let globals_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("web model globals"),
            contents: bytes_of(&globals),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let globals_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("web model globals layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let globals_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("web model globals bind group"),
            layout: &globals_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });
        let material_layout = create_material_layout(device);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("web model shader"),
            source: ShaderSource::Wgsl(include_str!("model.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("web model pipeline layout"),
            bind_group_layouts: &[&globals_layout, &material_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("web model pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::LAYOUT],
            },
            primitive: PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_FORMAT,
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
                targets: &[Some(ColorTargetState {
                    format: target_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });
        let scene = Scene::new(device, queue, model, &material_layout)?;

        Ok(Self {
            device: device.clone(),
            queue: queue.clone(),
            pipeline,
            globals_buffer,
            globals_bind_group,
            material_layout,
            depth,
            camera,
            scene,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
        self.depth = DepthTexture::new(&self.device, width, height);
    }

    fn render(&mut self, target: &TextureView, width: u32, height: u32, time_seconds: f32) {
        let expected_aspect = width as f32 / height.max(1) as f32;
        if (self.camera.aspect_ratio - expected_aspect).abs() > f32::EPSILON {
            self.resize(width, height);
        }

        let rotation = Mat4::from_rotation_y(time_seconds * 0.25);
        let transform = rotation
            * Mat4::from_scale(Vec3::splat(self.scene.scale))
            * Mat4::from_translation(-self.scene.center);
        let globals = Globals {
            view_projection: self.camera.view_projection_matrix().to_cols_array_2d(),
            model: transform.to_cols_array_2d(),
            light_direction: [-0.4, -1.0, -0.3, 0.0],
        };
        self.queue
            .write_buffer(&self.globals_buffer, 0, bytes_of(&globals));

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("web model encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("web model render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.018,
                            g: 0.024,
                            b: 0.04,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.globals_bind_group, &[]);
            for primitive in &self.scene.primitives {
                let material = &self.scene.materials[primitive.material];
                pass.set_bind_group(1, &material.bind_group, &[]);
                pass.set_vertex_buffer(0, primitive.mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(primitive.mesh.index_buffer.slice(..), IndexFormat::Uint32);
                pass.draw_indexed(0..primitive.mesh.index_count, 0, 0..1);
            }
        }
        self.queue.submit([encoder.finish()]);
    }
}

struct Scene {
    primitives: Vec<GpuPrimitive>,
    materials: Vec<GpuMaterial>,
    _textures: Vec<GpuTexture>,
    _fallback_texture: GpuTexture,
    center: Vec3,
    scale: f32,
}

impl Scene {
    fn new(
        device: &Device,
        queue: &Queue,
        model: &Model,
        material_layout: &BindGroupLayout,
    ) -> Result<Self, String> {
        if model.meshes.is_empty() {
            return Err("the GLB does not contain any triangle meshes".to_string());
        }

        let max_texture_size = device.limits().max_texture_dimension_2d;
        for image in &model.images {
            if image.width > max_texture_size || image.height > max_texture_size {
                return Err(format!(
                    "a texture is {}x{}, but this GPU supports at most {}x{}",
                    image.width, image.height, max_texture_size, max_texture_size,
                ));
            }
        }

        let textures = model
            .images
            .iter()
            .enumerate()
            .map(|(index, image)| {
                GpuTexture::from_rgba8(
                    device,
                    queue,
                    image.width,
                    image.height,
                    &image.pixels,
                    &format!("uploaded GLB texture {index}"),
                )
            })
            .collect::<Vec<_>>();
        let fallback_texture =
            GpuTexture::from_rgba8(device, queue, 1, 1, &[255, 255, 255, 255], "white texture");

        let materials = model
            .materials
            .iter()
            .enumerate()
            .map(|(index, material)| {
                let texture = material
                    .base_color_texture
                    .and_then(|texture_index| textures.get(texture_index))
                    .unwrap_or(&fallback_texture);
                GpuMaterial::new(device, material_layout, texture, material.base_color, index)
            })
            .collect::<Vec<_>>();

        let primitives = model
            .meshes
            .iter()
            .map(|mesh| GpuPrimitive {
                mesh: GpuMesh::new(device, &mesh.vertices, &mesh.indices),
                material: mesh.material.min(materials.len().saturating_sub(1)),
            })
            .collect::<Vec<_>>();

        let (center, scale) = model_bounds(model)?;
        Ok(Self {
            primitives,
            materials,
            _textures: textures,
            _fallback_texture: fallback_texture,
            center,
            scale,
        })
    }
}

struct GpuPrimitive {
    mesh: GpuMesh,
    material: usize,
}

struct GpuMaterial {
    bind_group: BindGroup,
    _uniform_buffer: Buffer,
}

impl GpuMaterial {
    fn new(
        device: &Device,
        layout: &BindGroupLayout,
        texture: &GpuTexture,
        base_color: [f32; 4],
        index: usize,
    ) -> Self {
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("material {index} uniform")),
            contents: bytes_of(&MaterialUniform { base_color }),
            usage: BufferUsages::UNIFORM,
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("material {index} bind group")),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });
        Self {
            bind_group,
            _uniform_buffer: uniform_buffer,
        }
    }
}

struct GpuTexture {
    _texture: Texture,
    view: TextureView,
    sampler: Sampler,
}

impl GpuTexture {
    fn from_rgba8(
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        pixels: &[u8],
        label: &str,
    ) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("GLB texture sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            _texture: texture,
            view,
            sampler,
        }
    }
}

struct DepthTexture {
    _texture: Texture,
    view: TextureView,
}

impl DepthTexture {
    fn new(device: &Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("web model depth texture"),
            size: Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        Self {
            _texture: texture,
            view,
        }
    }
}

fn create_material_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("web model material layout"),
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
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn model_bounds(model: &Model) -> Result<(Vec3, f32), String> {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for vertex in model.meshes.iter().flat_map(|mesh| &mesh.vertices) {
        let position = Vec3::from_array(vertex.position);
        if !position.is_finite() {
            return Err("the GLB contains non-finite vertex positions".to_string());
        }
        min = min.min(position);
        max = max.max(position);
    }

    if !min.is_finite() || !max.is_finite() {
        return Err("the GLB does not contain any vertices".to_string());
    }
    let largest_extent = (max - min).max_element();
    if largest_extent <= f32::EPSILON {
        return Err("the GLB geometry has no measurable size".to_string());
    }
    Ok(((min + max) * 0.5, 3.0 / largest_extent))
}

fn canvas_size(canvas: &HtmlCanvasElement) -> (u32, u32) {
    let dpr = web_sys::window()
        .expect("browser window unavailable")
        .device_pixel_ratio();
    let width = (canvas.client_width() as f64 * dpr).round().max(1.0) as u32;
    let height = (canvas.client_height() as f64 * dpr).round().max(1.0) as u32;
    (width, height)
}
