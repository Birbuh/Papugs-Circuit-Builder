use crate::mesh::Vertex;
use gltf::image::Format;
use wgpu::Texture;


#[derive(Debug)]
pub enum ModelLoadError {
    Gltf(gltf::Error),
    MissingPositions,
    AttributeCountMismatch,
    UnsupportedPrimitiveMode(gltf::mesh::Mode),
    UnsupportedImageFormat(gltf::image::Format),
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: usize,
}

pub struct Material {
    pub base_color: [f32; 4],
    pub base_color_texture: Option<usize>,
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>
}

pub struct Model {
    pub materials: Vec<Material>,
    pub meshes: Vec<Mesh>,
    pub images: Vec<Image>,
}

fn convert_image(image: &gltf::image::Data) -> Result<Image, ModelLoadError> {
    let pixels = match image.format {
        Format::R8G8B8A8 => image.pixels.clone(),
        Format::R8G8B8 => {
            let mut rgba = Vec::with_capacity(image.width as usize * image.height as usize * 4);
            for rgb in image.pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
            rgba
        }
        Format::R8G8 => {
            let mut rgba = Vec::with_capacity(image.width as usize * image.height as usize * 4);
            for values in image.pixels.chunks_exact(2) {
                rgba.extend_from_slice(&[values[0], values[0], values[0], values[1]]);
            }
            rgba
        }
        Format::R8 => {
            let mut rgba = Vec::with_capacity(image.width as usize * image.height as usize * 4);
            for &value in &image.pixels {
                rgba.extend_from_slice(&[value, value, value, 255]);
            }
            rgba
        }
        format => return Err(ModelLoadError::UnsupportedImageFormat(format))
    };
    Ok(Image {
        width: image.width,
        height: image.height,
        pixels,
    })
}


impl Model {
    pub fn from_glb_bytes(bytes: &[u8]) -> Result<Self, ModelLoadError>{
        let (document, buffers, gltf_images) = gltf::import_slice(bytes).map_err(ModelLoadError::Gltf)?;

        let images = gltf_images.iter().map(convert_image).collect::<Result<Vec<_>, _>>()?;

        let materials = document.materials().map(|material| {
            let pbr = material.pbr_metallic_roughness();
            let base_color = pbr.base_color_factor();
            let base_color_texture = pbr.base_color_texture().map(|info| {
                info.texture().source().index()
            });

            Material {
                base_color,
                base_color_texture,
            }
        }).collect::<Vec<_>>();

        let default_material = materials.len();
        let mut materials = materials;
        materials.push((Material { base_color: [1., 1., 1., 1.], base_color_texture: None, }));

        let mut meshes = Vec::new();

        for gltf_mesh in document.meshes() {
            for primitive in gltf_mesh.primitives() {
                if primitive.mode() != gltf::mesh::Mode::Triangles {
                    return Err(ModelLoadError::UnsupportedPrimitiveMode(primitive.mode()));
                }
                
                let reader = primitive.reader(|buffer| {
                    Some(buffers[buffer.index()].0.as_slice())
                });
                
                let positions = reader.read_positions().ok_or(ModelLoadError::MissingPositions)?.collect::<Vec<_>>();

                let normals = match reader.read_normals() {
                    Some(normals) => normals.collect::<Vec<_>>(),
                    None => vec![ [0., 1., 0.]; positions.len() ]
                };

                let uvs = match reader.read_tex_coords(0) {
                    Some(uvs) => uvs.into_f32().collect::<Vec<_>>(),
                    None => vec![ [0., 0.]; positions.len()]
                };

                if normals.len() != positions.len() || uvs.len() != positions.len() {
                    return Err(ModelLoadError::AttributeCountMismatch);
                }

                let vertices = positions.into_iter().zip(normals).zip(uvs).map(|((position, normal), uv)| {
                    Vertex {
                        position,
                        normal,
                        uv,
                    }
                }).collect::<Vec<_>>();

                let indices = match reader.read_indices() {
                    Some(indices) => indices.into_u32().collect::<Vec<_>>(),
                    None => (0..vertices.len() as u32).collect()
                };

                let material = primitive.material().index().unwrap_or(default_material);

                meshes.push(Mesh { vertices, indices, material, });
            }
        }
        Ok(Self { meshes, materials, images })
    }
}