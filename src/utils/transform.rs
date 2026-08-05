use crate::ds::model::TransformUniform;
use glam::{Mat4, Vec3, camera};

pub fn create_mvp_uniform_identity(
    cam_pos: Vec3,
    cam_look_at_pos: Vec3,
    cam_up: Vec3,
    vertical_fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
) -> TransformUniform {
    let model = Mat4::IDENTITY;
    let view = camera::rh::view::look_at_mat4(cam_pos, cam_look_at_pos, cam_up);

    // WGPU uses Direct X / WebGPU standard for NDC
    // Y-Up, X and Y between -1.0 and 1.0, z between 0.0 and 1.0
    let proj = camera::rh::proj::directx::perspective(vertical_fov, aspect_ratio, near, far);
    let transform = TransformUniform {
        model: model.to_cols_array_2d(),
        view: view.to_cols_array_2d(),
        proj: proj.to_cols_array_2d(),
    };

    transform
}
