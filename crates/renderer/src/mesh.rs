use bytemuck::{Pod, Zeroable, cast_slice};

use wgpu::{
    Buffer, BufferAddress, BufferUsages, Device, VertexAttribute, VertexBufferLayout, VertexStepMode, util::{BufferInitDescriptor, DeviceExt}, vertex_attr_array,
};

use std::mem::size_of;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2]
}

impl Vertex {
    pub const ATTRIBUTES: [VertexAttribute; 3] = vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];
    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &Self::ATTRIBUTES,
    };
    
}

pub struct GpuMesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
}

impl GpuMesh {
    pub fn new(device: &Device, vertices: &[Vertex], indices: &[u32]) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("mesh vertex buffer"),
            contents: cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("mesh index buffer"),
            contents: cast_slice(indices),
            usage: BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32
        }
    }
    
    // this one was copied from ChatGPT, I'm too lazy to write it all myself...
    // ########################################################################## begin
    pub const CUBE_VERTICES: &[Vertex] = &[
        // Front (+Z)
        Vertex {
            position: [-1.0, -1.0,  1.0],
            normal:   [ 0.0,  0.0,  1.0],
            uv:       [ 0.0,  1.0],
        },
        Vertex {
            position: [ 1.0, -1.0,  1.0],
            normal:   [ 0.0,  0.0,  1.0],
            uv:       [ 1.0,  1.0],
        },
        Vertex {
            position: [ 1.0,  1.0,  1.0],
            normal:   [ 0.0,  0.0,  1.0],
            uv:       [ 1.0,  0.0],
        },
        Vertex {
            position: [-1.0,  1.0,  1.0],
            normal:   [ 0.0,  0.0,  1.0],
            uv:       [ 0.0,  0.0],
        },
    
        // Back (-Z)
        Vertex {
            position: [ 1.0, -1.0, -1.0],
            normal:   [ 0.0,  0.0, -1.0],
            uv:       [ 0.0,  1.0],
        },
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal:   [ 0.0,  0.0, -1.0],
            uv:       [ 1.0,  1.0],
        },
        Vertex {
            position: [-1.0,  1.0, -1.0],
            normal:   [ 0.0,  0.0, -1.0],
            uv:       [ 1.0,  0.0],
        },
        Vertex {
            position: [ 1.0,  1.0, -1.0],
            normal:   [ 0.0,  0.0, -1.0],
            uv:       [ 0.0,  0.0],
        },
    
        // Right (+X)
        Vertex {
            position: [1.0, -1.0,  1.0],
            normal:   [1.0,  0.0,  0.0],
            uv:       [0.0, 1.0],
        },
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal:   [1.0,  0.0,  0.0],
            uv:       [1.0, 1.0],
        },
        Vertex {
            position: [1.0,  1.0, -1.0],
            normal:   [1.0,  0.0,  0.0],
            uv:       [1.0, 0.0],
        },
        Vertex {
            position: [1.0,  1.0,  1.0],
            normal:   [1.0,  0.0,  0.0],
            uv:       [0.0, 0.0],
        },
    
        // Left (-X)
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal:   [-1.0,  0.0,  0.0],
            uv:       [0.0, 1.0],
        },
        Vertex {
            position: [-1.0, -1.0,  1.0],
            normal:   [-1.0,  0.0,  0.0],
            uv:       [1.0, 1.0],
        },
        Vertex {
            position: [-1.0,  1.0,  1.0],
            normal:   [-1.0,  0.0,  0.0],
            uv:       [1.0, 0.0],
        },
        Vertex {
            position: [-1.0,  1.0, -1.0],
            normal:   [-1.0,  0.0,  0.0],
            uv:       [0.0, 0.0],
        },
    
        // Top (+Y)
        Vertex {
            position: [-1.0, 1.0,  1.0],
            normal:   [ 0.0, 1.0,  0.0],
            uv:       [ 0.0, 1.0],
        },
        Vertex {
            position: [ 1.0, 1.0,  1.0],
            normal:   [ 0.0, 1.0,  0.0],
            uv:       [ 1.0, 1.0],
        },
        Vertex {
            position: [ 1.0, 1.0, -1.0],
            normal:   [ 0.0, 1.0,  0.0],
            uv:       [ 1.0, 0.0],
        },
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal:   [ 0.0, 1.0,  0.0],
            uv:       [ 0.0, 0.0],
        },
    
        // Bottom (-Y)
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal:   [ 0.0, -1.0,  0.0],
            uv:       [ 0.0, 1.0],
        },
        Vertex {
            position: [ 1.0, -1.0, -1.0],
            normal:   [ 0.0, -1.0,  0.0],
            uv:       [ 1.0, 1.0],
        },
        Vertex {
            position: [ 1.0, -1.0,  1.0],
            normal:   [ 0.0, -1.0,  0.0],
            uv:       [ 1.0, 0.0],
        },
        Vertex {
            position: [-1.0, -1.0,  1.0],
            normal:   [ 0.0, -1.0,  0.0],
            uv:       [ 0.0, 0.0],
        },
    ];
    
    pub const CUBE_INDICES: &[u32] = &[
         0,  1,  2,
         0,  2,  3,
    
         4,  5,  6,
         4,  6,  7,
    
         8,  9, 10,
         8, 10, 11,
    
        12, 13, 14,
        12, 14, 15,
    
        16, 17, 18,
        16, 18, 19,
    
        20, 21, 22,
        20, 22, 23,
    ];
}
// ########################################################################## end