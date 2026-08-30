struct Globals {
    view_projection: mat4x4<f32>,
    model: mat4x4<f32>,
    light_direction: vec4<f32>,
};

struct Material {
    base_color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;

@group(1) @binding(1)
var base_color_sampler: sampler;

@group(1) @binding(2)
var<uniform> material: Material;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_position = globals.model * vec4<f32>(input.position, 1.0);
    output.clip_position = globals.view_projection * world_position;
    output.world_normal = normalize((globals.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(input.world_normal);
    let light = normalize(-globals.light_direction.xyz);
    let diffuse = max(dot(normal, light), 0.0);
    let brightness = 0.22 + diffuse * 0.78;
    let texture_color = textureSample(base_color_texture, base_color_sampler, input.uv);
    let color = texture_color * material.base_color;
    return vec4<f32>(color.rgb * brightness, color.a);
}
