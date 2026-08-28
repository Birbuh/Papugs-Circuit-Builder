use glam::{Mat4, Vec3, camera::rh::{view::look_at_mat4, proj::directx::perspective}};

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,

    pub fov_y: f32,
    pub aspect_ratio: f32,

    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            position: Vec3::new(4., 3., 5.),
            target: Vec3::ZERO,
            
            fov_y: 60_f32.to_radians(),
            aspect_ratio: width as f32 / height.max(1) as f32, // x/0 isn't valid...

            near: 0.1,
            far: 100.,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32  / height.max(1) as f32 // x/0 isn't valid...
    }

    pub fn view_matrix(&self) -> Mat4 {
        look_at_mat4(self.position, self.target, Vec3::Y)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        perspective(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}