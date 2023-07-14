use crate::{
    camera3d::Camera3d,
    geometry::{plane::PlaneSectionVertex, Vertex},
    Assets,
};

use geng::prelude::*;

pub fn draw_cross_section(
    cross_section: &[PlaneSectionVertex],
    color: Rgba<f32>,
    camera: &Camera3d,
    assets: &Assets,
    geng: &Geng,
    framebuffer: &mut ugli::Framebuffer,
) {
    let framebuffer_size = framebuffer.size().map(|x| x as f32);

    let geometry = cross_section
        .iter()
        .map(|v| Vertex {
            a_pos: v.world_pos,
            a_normal: vec3::ZERO,
            a_color: Rgba::WHITE,
        })
        .collect();
    let geometry = ugli::VertexBuffer::new_dynamic(geng.ugli(), geometry);

    ugli::draw(
        framebuffer,
        &assets.cross.get(),
        ugli::DrawMode::TriangleFan,
        &geometry,
        (
            ugli::uniforms! {
                u_model_matrix: mat4::identity(),
                u_color: color,
            },
            camera.uniforms(framebuffer_size),
        ),
        ugli::DrawParameters {
            // cull_face: Some(ugli::CullFace::Back),
            depth_func: Some(ugli::DepthFunc::Less),
            ..Default::default()
        },
    );

    // Highlight the vertices
    let vertex_geometry = geng_utils::geometry::unit_quad_geometry(geng.ugli());
    for v in cross_section {
        let matrix = mat4::translate(v.world_pos) * mat4::scale_uniform(0.1);
        ugli::draw(
            framebuffer,
            &assets.simple3d.get(),
            ugli::DrawMode::TriangleFan,
            &vertex_geometry,
            (
                ugli::uniforms! {
                    u_model_matrix: matrix,
                    u_color: Rgba::CYAN,
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters {
                // cull_face: Some(ugli::CullFace::Back),
                // depth_func: Some(ugli::DepthFunc::Less),
                ..Default::default()
            },
        );
    }
}
