struct TransformUniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>};

struct LightingUniforms {
    wc_light_direction: vec3<f32>,
    sun_light_energy: f32,
    ambient_light_contribution: f32}

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

// Texture
//binding group 3, resource slot  0
@group(3) @binding(0)
var spl: sampler;
//binding group 3, resource slot 1
@group(3) @binding(1)
var t_diffuse: texture_2d<f32>;
//binding group 3, resource slot 2
@group(3) @binding(2)
var t_normal: texture_2d<f32>;
//binding group 3, resource slot 3
@group(3) @binding(3)
var t_specular: texture_2d<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>};

struct V2F {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) view_position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) view_normal: vec3<f32>}

@vertex
fn vs_main(vertex: VertexInput) -> V2F {
    var output: V2F;

    let world_position = transforms.model * vec4<f32>(vertex.position, 1.0);
    output.world_position = world_position.xyz;

    let view_position = transforms.view * world_position;
    output.view_position = view_position.xyz;
    output.view_normal = (transforms.view * vec4<f32>(vertex.normal, 0.0)).xyz;

    output.clip_position = transforms.proj * view_position;
    output.uv = vertex.uv;
    return output;
}

@fragment
fn fs_main(interp: V2F, @builtin(front_facing) is_front_facing: bool) -> @location(0) vec4<f32> {
    let shininess = material.shininess;

    // build tangent frame using cotangent frame method: http://www.thetenthplanet.de/archives/1180
    let view_light_dir = normalize((transforms.view * vec4<f32>(light.wc_light_direction, 0.0)).xyz);
    let N = normalize(interp.view_normal);

    let dp1 = dpdx(interp.view_position);
    let dp2 = dpdy(interp.view_position);
    let duv1 = dpdx(interp.uv);
    let duv2 = dpdy(interp.uv);

    let dp2perp = cross(dp2, N);
    let dp1perp = cross(N, dp1);

    let T = normalize(dp2perp * duv1.x + dp1perp * duv2.x);
    let B = normalize(dp2perp * duv1.y + dp1perp * duv2.y);

    var normal_color = textureSample(t_normal, spl, interp.uv).xyz;
    normal_color = normal_color * 2.0 - 1.0; // map normal range to -1 to 1

    var view_perturb_normal = normal_color.x * T + normal_color.y * B + normal_color.z * N;

    // assume base color is packed in big-endian RGBA format
    let dr = f32((material.k_diffuse >> 24) & 0xFF) / 255.0;
    let dg = f32((material.k_diffuse >> 16) & 0xFF) / 255.0;
    let db = f32((material.k_diffuse >> 8) & 0xFF) / 255.0;
    let da = f32(material.k_diffuse & 0xFF) / 255.0;

    let sr = f32((material.k_specular >> 24) & 0xFF) / 255.0;
    let sg = f32((material.k_specular >> 16) & 0xFF) / 255.0;
    let sb = f32((material.k_specular >> 8) & 0xFF) / 255.0;
    let sa = f32(material.k_specular & 0xFF) / 255.0;

    let diffuse_contribution = vec4<f32>(dr, dg, db, da);
    let specular_contribution = vec4<f32>(sr, sg, sb, sa);

    let diffuse_color = textureSample(t_diffuse, spl, interp.uv) * diffuse_contribution;
    let specular_color = textureSample(t_specular, spl, interp.uv) * specular_contribution;

    let ambient_term = vec4<f32>(diffuse_color.rgb * light.sun_light_energy * light.ambient_light_contribution, diffuse_color.a);
    let diffuse_term = vec4<f32>(diffuse_color.rgb * light.sun_light_energy * max(0, dot(view_perturb_normal, (-view_light_dir))), diffuse_color.a);

    let half_vec = normalize((normalize(-interp.view_position) - view_light_dir));
    let specular_term = vec4<f32>(specular_color.rgb * light.sun_light_energy * pow(max(0, dot(view_perturb_normal, half_vec)), shininess), specular_color.a);

    return clamp(ambient_term + diffuse_term + specular_term, vec4f(0.0), vec4f(1.0));
}
