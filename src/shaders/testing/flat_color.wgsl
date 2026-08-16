struct TransformUniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>};

struct LightingUniforms {
    wc_light_direction: vec3<f32>}

struct MaterialUniforms {
    k_diffuse: u32,
    k_specular: u32,
    k_emissive: u32,
    index_of_refraction: f32,
    shininess: f32,
    dissolve: f32,
    illumination_model: f32}

// uniform passed from CPU
// Transform
// binding group 0, resource slot 0
@group(0) @binding(0)
var<uniform> transforms: TransformUniforms;

// Light
// binding group 1, resource slot 0
@group(1) @binding(0)
var<uniform> light: LightingUniforms;

// Material
// binding group 2, resource slot 0
@group(2) @binding(0)
var<uniform> material: MaterialUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>};

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

    let light_intensity = max(dot(normal, normalize(light.wc_light_direction)), 0.0);

    // assume base color is packed in big-endian RGBA format
    let r = f32((material.k_diffuse >> 24) & 0xFF) / 255.0;
    let g = f32((material.k_diffuse >> 16) & 0xFF) / 255.0;
    let b = f32((material.k_diffuse >> 8) & 0xFF) / 255.0;
    let a = f32(material.k_diffuse & 0xFF) / 255.0;

    return vec4<f32>(vec3<f32>(r, g, b) * light_intensity, a);
}
