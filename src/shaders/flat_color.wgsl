struct TransformUniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>};

struct LightingUniforms {
    wc_light_direction: vec3<f32>}

struct MaterialUniforms {
    base_color: u32}

// uniform passed from CPU
// binding group 0, resource slot 0
@group(0) @binding(0)
var<uniform> transforms: TransformUniforms;

// binding group 1, resource slot 0
@group(1) @binding(0)
var<uniform> light: LightingUniforms;

// binding group 1, resource slot 1
@group(1) @binding(1)
var<uniform> material: MaterialUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct V2F {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) view_position: vec3<f32>};

@vertex
fn vs_main(vertex: VertexInput) -> V2F {
    var output: V2F;

    let world_position = transforms.model * vec4<f32>(vertex.position, 1.0);
    output.world_position = world_position.xyz;

    let view_position = transforms.view * world_position;
    output.view_position = view_position.xyz;

    output.clip_position = transforms.proj * view_position;
    return output;
}

@fragment
fn fs_main(interp: V2F, @builtin(front_facing) is_front_facing: bool) -> @location(0) vec4<f32> {
    let dx = dpdx(interp.world_position);
    let dy = dpdy(interp.world_position);

    // assuming wc coordinate is in X-Right + Y-Up coordinate system with Z-Out
    var normal = normalize(cross(dx, dy));

    // flip the normal if the face is facing away from the camera
    if !is_front_facing {
        normal = -normal;
    }

    let light_intensity = dot(normal, normalize(light.wc_light_direction));

    // assume base color is packed in big-endian RGBA format
    let r = f32((material.base_color >> 24) & 0xFF) / 255.0;
    let g = f32((material.base_color >> 16) & 0xFF) / 255.0;
    let b = f32((material.base_color >> 8) & 0xFF) / 255.0;
    let a = f32(material.base_color & 0xFF) / 255.0;

    return vec4<f32>(r, g, b, a) * light_intensity;
}
