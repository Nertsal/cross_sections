mod mat_5;
pub mod plane;
pub mod shape;
pub mod space;
mod vec_5;

pub use self::mat_5::mat5;
pub use self::vec_5::vec5;

use geng::prelude::*;

pub fn vec4_len<T: Float>(v: vec4<T>) -> T {
    vec4::dot(v, v).sqrt()
}

pub fn vec4_norm<T: Float>(v: vec4<T>) -> vec4<T> {
    let len = vec4_len(v);
    if len.approx_eq(&T::ZERO) {
        return vec4::ZERO;
    }
    v / len
}

#[derive(ugli::Vertex, Debug, Clone, Copy)]
pub struct Vertex {
    pub a_pos: vec3<f32>,
    pub a_normal: vec3<f32>,
    pub a_color: Rgba<f32>,
}

impl Vertex {
    pub fn new(pos: vec3<f32>, normal: vec3<f32>, color: Rgba<f32>) -> Self {
        Self {
            a_pos: pos,
            a_normal: normal,
            a_color: color,
        }
    }

    pub fn new_pos(pos: vec3<f32>) -> Self {
        Self::new(pos, vec3::ZERO, Rgba::WHITE)
    }

    pub fn white(pos: vec3<f32>, normal: vec3<f32>) -> Self {
        Self::new(pos, normal, Rgba::WHITE)
    }

    pub fn colored(self, color: Rgba<f32>) -> Self {
        Self {
            a_color: color,
            ..self
        }
    }
}
