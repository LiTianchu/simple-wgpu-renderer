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

#[derive(Debug, Clone, Copy)]
pub struct CameraInfo {
    pub position: Vec3,
    pub look_at: Vec3,
    pub up: Vec3,
    pub fov: f32,
}

impl Default for CameraInfo {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            look_at: Vec3::ZERO,
            up: Vec3::Y,
            fov: 45.0,
        }
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
