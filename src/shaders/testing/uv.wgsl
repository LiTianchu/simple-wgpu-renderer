struct TransformUniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>};

// uniform passed from CPU
// binding group 0, resource slot 0
@group(0) @binding(0)
var<uniform> transforms: TransformUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct V2F {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) view_position: vec3<f32>,
    @location(2) uv: vec2<f32>};

@vertex
fn vs_main(vertex: VertexInput) -> V2F {
    var output: V2F;

    let world_position = transforms.model * vec4<f32>(vertex.position, 1.0);
    output.world_position = world_position.xyz;

    let view_position = transforms.view * world_position;
    output.view_position = view_position.xyz;

    output.clip_position = transforms.proj * view_position;
    output.uv = vertex.uv;
    return output;
}

@fragment
fn fs_main(interp: V2F, @builtin(front_facing) is_front_facing: bool) -> @location(0) vec4<f32> {
    return vec4<f32>(interp.uv.x, interp.uv.y, 0.0, 1.0);
}
