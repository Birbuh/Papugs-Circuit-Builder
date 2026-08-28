// this file is vibe-coded, I don't know this language.

struct Globals {
    view_projection: mat4x4<f32>,
    model: mat4x4<f32>,

    light_direction: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> globals: Globals;


struct VertexInput {
    @location(0)
    position: vec3<f32>,

    @location(1)
    normal: vec3<f32>,
};


struct VertexOutput {
    @builtin(position)
    clip_position: vec4<f32>,

    @location(0)
    world_position: vec3<f32>,

    @location(1)
    world_normal: vec3<f32>,
};


@vertex
fn vs_main(
    input: VertexInput
) -> VertexOutput {
    var output: VertexOutput;

    let world_position =
        globals.model
        * vec4<f32>(
            input.position,
            1.0
        );

    output.clip_position =
        globals.view_projection
        * world_position;

    output.world_position =
        world_position.xyz;

    // Correct for rotation and uniform scaling.
    //
    // Later, when supporting arbitrary non-uniform
    // scales, pass a proper inverse-transpose
    // normal matrix instead.
    output.world_normal =
        normalize(
            (
                globals.model
                * vec4<f32>(
                    input.normal,
                    0.0
                )
            ).xyz
        );

    return output;
}


@fragment
fn fs_main(
    input: VertexOutput
) -> @location(0) vec4<f32> {
    let normal =
        normalize(input.world_normal);

    let light =
        normalize(
            -globals.light_direction.xyz
        );

    let diffuse =
        max(
            dot(normal, light),
            0.0
        );

    let ambient = 0.18;

    let base_color =
        vec3<f32>(
            0.12,
            0.48,
            0.90
        );

    let brightness =
        ambient
        + diffuse * 0.82;

    return vec4<f32>(
        base_color * brightness,
        1.0
    );
}