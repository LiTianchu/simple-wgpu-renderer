use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct ObjectTransform {
    pub translation: Vec3,
    pub rotation: glam::Quat,
    pub scale: Vec3,
}

impl Default for ObjectTransform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}
impl ObjectTransform {
    pub fn set_rotation_euler(&mut self, euler: Vec3) {
        self.rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, euler.x, euler.y, euler.z);
    }

    pub fn set_scale_uniform(&mut self, scale: f32) {
        self.scale = Vec3::new(scale, scale, scale);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CameraInfo {
    pub position: Vec3,
    pub look_at: Vec3,
    pub up: Vec3,
    pub fov: f32, // vertical_fov in radians
}

impl Default for CameraInfo {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            look_at: Vec3::ZERO,
            up: Vec3::Y,
            fov: 45.0_f32.to_radians(),
        }
    }
}

impl CameraInfo {
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn set_look_at(&mut self, look_at: Vec3) {
        self.look_at = look_at;
    }

    pub fn set_up(&mut self, up: Vec3) {
        self.up = up;
    }

    pub fn set_fov_degrees(&mut self, fov_degrees: f32) {
        self.fov = fov_degrees.to_radians();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectionInfo {
    pub near: f32,
    pub far: f32,
}

impl Default for ProjectionInfo {
    fn default() -> Self {
        Self {
            near: 1.0,
            far: 1000.0,
        }
    }
}
