use geng::prelude::*;

#[derive(Debug, Clone)]
pub struct Camera3d {
    pub fov: Angle<f32>,
    pub pos: vec3<f32>,
    pub rot_h: Angle<f32>,
    pub rot_v: Angle<f32>,
    pub near: f32,
    pub far: f32,
}

impl Camera3d {
    pub fn look_dir(&self) -> vec3<f32> {
        (mat4::rotate_x(self.rot_v) * mat4::rotate_y(self.rot_h) * (-vec3::UNIT_Z).extend(1.0))
            .into_3d()
    }
}

impl geng::AbstractCamera3d for Camera3d {
    fn view_matrix(&self) -> mat4<f32> {
        mat4::rotate_x(-self.rot_v)
            * mat4::rotate_y(-self.rot_h)
            // * mat4::rotate_x(Angle::from_radians(-f32::PI / 2.0))
            * mat4::translate(-self.pos)
    }

    fn projection_matrix(&self, framebuffer_size: vec2<f32>) -> mat4<f32> {
        mat4::perspective(
            self.fov.as_radians(),
            framebuffer_size.x / framebuffer_size.y,
            self.near,
            self.far,
        )
    }
}
