use super::Object;

use crate::{camera3d::Camera3d, Assets};

use geng::prelude::*;

pub fn draw_cut(
    obj: &Object,
    cutoff_matrix: mat4<f32>,
    camera: &Camera3d,
    assets: &Assets,
    framebuffer: &mut ugli::Framebuffer,
) {
    let framebuffer_size = framebuffer.size().map(|x| x as f32);

    let matrix = obj.matrix();

    ugli::draw(
        framebuffer,
        &assets.cut_front.get(),
        ugli::DrawMode::Triangles,
        &*obj.geometry,
        (
            ugli::uniforms! {
                u_model_matrix: matrix,
                u_cutoff_matrix: cutoff_matrix,
                u_color: Rgba::opaque(0.6, 0.6, 0.6),
            },
            camera.uniforms(framebuffer_size),
        ),
        ugli::DrawParameters {
            cull_face: Some(ugli::CullFace::Back),
            depth_func: Some(ugli::DepthFunc::Less),
            ..Default::default()
        },
    );
    ugli::draw(
        framebuffer,
        &assets.cut_back.get(),
        ugli::DrawMode::Triangles,
        &*obj.geometry,
        (
            ugli::uniforms! {
                u_model_matrix: matrix,
                u_cutoff_matrix: cutoff_matrix,
                u_color: obj.color,
            },
            camera.uniforms(framebuffer_size),
        ),
        ugli::DrawParameters {
            cull_face: Some(ugli::CullFace::Front),
            depth_func: Some(ugli::DepthFunc::Less),
            ..Default::default()
        },
    );
}
