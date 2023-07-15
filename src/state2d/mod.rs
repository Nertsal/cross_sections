mod cross;
mod cut;

use crate::{
    camera3d::Camera3d,
    geometry::{
        plane::{Plane, PlaneSectionVertex},
        shape::Triangle,
        Vertex,
    },
    Assets,
};

use geng::prelude::*;
use geng_utils::{conversions::Vec2RealConversions, key as key_utils, texture as texture_utils};

pub struct Object {
    pub geometry: Rc<ugli::VertexBuffer<Vertex>>,
    pub position: vec3<f32>,
    pub orientation: vec3<f32>,
    pub roll: Angle<f32>,
    pub scale: f32,
    pub color: Rgba<f32>,
}

impl Object {
    pub fn new(position: vec3<f32>, geometry: Rc<ugli::VertexBuffer<Vertex>>) -> Self {
        Self {
            geometry,
            position,
            orientation: vec3::UNIT_X,
            roll: Angle::ZERO,
            scale: 1.0,
            color: Rgba::WHITE,
        }
    }

    pub fn matrix(&self) -> mat4<f32> {
        let flat = vec2(self.orientation.x, self.orientation.z);
        let rot_h = flat.arg();
        let rot_v = vec2(flat.len(), self.orientation.y).arg();
        mat4::translate(self.position)
            * mat4::rotate_x(self.roll)
            * mat4::rotate_z(-rot_v)
            * mat4::rotate_y(rot_h)
            * mat4::scale_uniform(self.scale)
    }

    pub fn rotate_y(&mut self, angle: Angle<f32>) {
        let flat = vec2(self.orientation.x, self.orientation.z);
        let flat = flat.rotate(angle);
        self.orientation = vec3(flat.x, self.orientation.y, flat.y);
    }
}

enum Drag {
    X,
    Y,
}

pub struct State2d {
    geng: Geng,
    assets: Rc<Assets>,
    framebuffer_size: vec2<usize>,
    unit_geometry: Rc<ugli::VertexBuffer<draw2d::TexturedVertex>>,
    cut_texture: ugli::Texture,
    cut_depth: ugli::Renderbuffer<ugli::DepthComponent>,
    cut_post_texture: ugli::Texture,
    cross_texture: ugli::Texture,
    cross_depth: ugli::Renderbuffer<ugli::DepthComponent>,
    flat_texture: ugli::Texture,
    lower_left_size: vec2<f32>,
    camera3d: Camera3d,
    camera2d: Camera2d,
    simulation_time: f32,
    next_spawn: f32,
    prefabs: Vec<Rc<ugli::VertexBuffer<Vertex>>>,
    objects: Vec<Object>,
    cursor_pos: vec2<f32>,
    separator_x: Aabb2<f32>,
    separator_y: Aabb2<f32>,
    drag: Option<Drag>,
}

impl State2d {
    pub fn new(geng: Geng, assets: Rc<Assets>) -> Self {
        let prefab = |geometry| Rc::new(ugli::VertexBuffer::new_dynamic(geng.ugli(), geometry));
        Self {
            framebuffer_size: vec2(1, 1),
            unit_geometry: Rc::new(geng_utils::geometry::unit_quad_geometry(geng.ugli())),
            cut_texture: texture_utils::new_texture(geng.ugli(), vec2(1, 1)),
            cut_depth: ugli::Renderbuffer::new(geng.ugli(), vec2(1, 1)),
            cut_post_texture: texture_utils::new_texture(geng.ugli(), vec2(1, 1)),
            cross_texture: texture_utils::new_texture(geng.ugli(), vec2(1, 1)),
            cross_depth: ugli::Renderbuffer::new(geng.ugli(), vec2(1, 1)),
            flat_texture: texture_utils::new_texture(geng.ugli(), vec2(1, 1)),
            lower_left_size: vec2(0.4, 0.5),
            camera3d: Camera3d {
                fov: Angle::from_radians(70.0),
                pos: vec3(6.0, 0.0, 10.0),
                rot_h: Angle::from_degrees(30.0),
                rot_v: Angle::ZERO,
                near: 0.1,
                far: 1000.0,
            },
            camera2d: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            simulation_time: 0.0,
            next_spawn: 0.0,
            prefabs: vec![prefab(crate::geometry::shape::unit_cube_triangulated())],
            objects: Vec::new(),
            cursor_pos: vec2::ZERO,
            separator_x: Aabb2::ZERO,
            separator_y: Aabb2::ZERO,
            drag: None,
            geng,
            assets,
        }
    }

    fn random_spawn(&self) -> vec2<f32> {
        let mut rng = thread_rng();
        let x = rng.gen_range(-1.0..=1.0);
        let y = rng.gen_range(-1.0..=1.0);
        let pos = vec2(x, y);
        let pos = (self
            .camera2d
            .projection_matrix(self.flat_texture.size().as_f32())
            * self.camera2d.view_matrix())
        .inverse()
            * pos.extend(1.0);
        pos.into_2d()
    }

    pub fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        let config = self.assets.config.get();

        self.simulation_time += delta_time;
        self.next_spawn -= delta_time;
        let mut rng = thread_rng();
        while self.next_spawn < 0.0 {
            if self.objects.len() >= config.object_limit {
                self.next_spawn = 1.0;
                break;
            }

            // Final range: 0.1..=0.5
            self.next_spawn += rng.gen_range(0.0..=1.0).sqr() * 0.4 + 0.1;
            if let Some(geometry) = self.prefabs.choose(&mut rng) {
                let scale = rng.gen_range(config.scale_min..=config.scale_max);
                let pos_z = -scale * 2.0;

                let pos = 'outer: {
                    let mut pos = self.random_spawn().extend(pos_z);
                    for _ in 0..5 {
                        let mut good = true;
                        for obj in &self.objects {
                            let dist = (pos - obj.position).len();
                            if dist < (scale + obj.scale) * 1.74 {
                                // Try another one
                                pos = self.random_spawn().extend(pos_z);
                                good = false;
                                break;
                            }
                        }
                        if good {
                            break 'outer Some(pos);
                        }
                    }
                    None
                };

                if let Some(pos) = pos {
                    let mut obj = Object::new(pos, geometry.clone());
                    obj.orientation = vec3(
                        rng.gen_range(-1.0..=1.0),
                        rng.gen_range(-1.0..=1.0),
                        rng.gen_range(-1.0..=1.0),
                    );
                    obj.roll = Angle::from_degrees(rng.gen_range(0.0..=360.0));
                    obj.scale = scale;
                    obj.color = config
                        .object_colors
                        .choose(&mut rng)
                        .copied()
                        .unwrap_or(Rgba::WHITE);
                    self.objects.push(obj);
                }
            }
        }

        for obj in &mut self.objects {
            obj.position += vec3::UNIT_Z * config.speed * delta_time;
            obj.rotate_y(Angle::from_degrees(
                config.rotation_speed_degrees * delta_time,
            ));
        }
        // Delete far objects
        self.objects.retain(|obj| obj.position.z < 5.0);
    }

    pub fn handle_event(&mut self, event: &geng::Event) {
        if let geng::Event::TouchMove(touch) = event {
            self.touch_move(touch.position.as_f32());
        }
        if let geng::Event::CursorMove { position } = event {
            self.touch_move(position.as_f32());
        }

        // TODO: multitouch
        if let geng::Event::TouchStart(touch) = event {
            self.touch_press(touch.position.as_f32());
        }
        if let geng::Event::TouchEnd(_) = event {
            self.touch_release();
        }

        if key_utils::is_event_press(event, [geng::MouseButton::Left]) {
            self.touch_press(self.cursor_pos);
        } else if key_utils::is_event_release(event, [geng::MouseButton::Left]) {
            self.touch_release();
        }
    }

    fn touch_press(&mut self, pos: vec2<f32>) {
        self.cursor_pos = pos;
        if self.separator_x.contains(self.cursor_pos) {
            self.drag = Some(Drag::X);
        } else if self.separator_y.contains(self.cursor_pos) {
            self.drag = Some(Drag::Y);
        } else {
            self.drag = None;
        }
    }

    fn touch_move(&mut self, pos: vec2<f32>) {
        self.cursor_pos = pos;
        if let Some(drag) = &self.drag {
            match drag {
                Drag::X => {
                    self.lower_left_size.x = self.cursor_pos.x / self.framebuffer_size.x as f32;
                }
                Drag::Y => {
                    self.lower_left_size.y = self.cursor_pos.y / self.framebuffer_size.y as f32;
                }
            }
        }
    }

    fn touch_release(&mut self) {
        self.drag = None;
    }

    pub fn draw(&mut self, include_3d: bool, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        let config = self.assets.config.get();
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);

        let framebuffer_size = framebuffer.size().as_f32();

        // Layout
        let cut_pos = Aabb2::ZERO.extend_positive(self.lower_left_size * framebuffer_size);
        let cross_pos = Aabb2::point(vec2(0.0, cut_pos.max.y)).extend_positive(
            vec2(self.lower_left_size.x, 1.0 - self.lower_left_size.y) * framebuffer_size,
        );
        let flat_pos = if include_3d {
            Aabb2::point(vec2(cut_pos.max.x, 0.0))
                .extend_positive(vec2(1.0 - self.lower_left_size.x, 1.0) * framebuffer_size)
        } else {
            Aabb2::ZERO.extend_positive(framebuffer_size)
        };

        // Update textures
        let texture_size = |pos: Aabb2<f32>| pos.size().map(|x| (x.round() as usize).max(1));
        if include_3d {
            let cut_size = texture_size(cut_pos);
            if self.cut_texture.size() != cut_size {
                self.cut_depth = ugli::Renderbuffer::new(self.geng.ugli(), cut_size);
            }
            texture_utils::update_texture_size(&mut self.cut_texture, cut_size, self.geng.ugli());
            texture_utils::update_texture_size(
                &mut self.cut_post_texture,
                cut_size,
                self.geng.ugli(),
            );

            let cross_size = texture_size(cross_pos);
            if self.cross_texture.size() != cross_size {
                self.cross_depth = ugli::Renderbuffer::new(self.geng.ugli(), cross_size);
            }
            texture_utils::update_texture_size(
                &mut self.cross_texture,
                cross_size,
                self.geng.ugli(),
            );
        }
        texture_utils::update_texture_size(
            &mut self.flat_texture,
            texture_size(flat_pos),
            self.geng.ugli(),
        );

        // Calculate a cross section
        let cross_plane = Plane {
            normal: vec3(0.0, 0.0, 1.0),
            offset: 0.0,
        };
        let cross_sections: Vec<(usize, Vec<PlaneSectionVertex>)> = self
            .objects
            .iter()
            .enumerate()
            .flat_map(|(i, obj)| {
                let a = obj.geometry.iter().step_by(3);
                let b = obj.geometry.iter().skip(1).step_by(3);
                let c = obj.geometry.iter().skip(2).step_by(3);
                let transform = |v: vec3<f32>| (obj.matrix() * v.extend(1.0)).into_3d();
                let triangles = itertools::izip![a, b, c].map(|(a, b, c)| {
                    Triangle::new([transform(a.a_pos), transform(b.a_pos), transform(c.a_pos)])
                });
                let cross_section = cross_plane.cross_sect(triangles);
                (cross_section.len() >= 3).then_some((i, cross_section))
            })
            .collect();

        // Render
        if include_3d {
            // Cut out the part in front of the plane
            let mut cut_buffer = attach_clear(
                &mut self.cut_texture,
                &mut self.cut_depth,
                config.background_color,
                self.geng.ugli(),
            );
            for obj in &self.objects {
                cut::draw_cut(
                    obj,
                    cross_plane.matrix(),
                    &self.camera3d,
                    &self.assets,
                    &mut cut_buffer,
                );
            }

            // Outlines
            let mut post_buffer =
                texture_utils::attach_texture(&mut self.cut_post_texture, self.geng.ugli());
            ugli::draw(
                &mut post_buffer,
                &self.assets.postprocess.get(),
                ugli::DrawMode::TriangleFan,
                &*self.unit_geometry,
                ugli::uniforms! {
                    u_color_texture: &self.cut_texture,
                    u_outline_texture: &self.cut_texture,
                    u_outline_texture_size: self.cut_texture.size(),
                    u_outline_color: Rgba::BLACK,
                },
                ugli::DrawParameters { ..default() },
            );
            draw_texture_to(&self.cut_post_texture, cut_pos, &self.geng, framebuffer);

            // Draw only the cross section in 3d
            let mut cross_buffer = attach_clear(
                &mut self.cross_texture,
                &mut self.cross_depth,
                config.background_color,
                self.geng.ugli(),
            );
            for (i, cross_section) in &cross_sections {
                let i = *i;
                cross::draw_cross_section(
                    cross_section,
                    self.objects[i].color,
                    &self.camera3d,
                    &self.assets,
                    &self.geng,
                    &mut cross_buffer,
                );
            }
            draw_texture_to(&self.cross_texture, cross_pos, &self.geng, framebuffer);
        }

        // Draw the cross section in 2d
        let mut flat_buffer =
            texture_utils::attach_texture(&mut self.flat_texture, self.geng.ugli());
        ugli::clear(&mut flat_buffer, Some(config.background_color), None, None);
        for (i, cross_section) in &cross_sections {
            let i = *i;
            draw_flat_section(
                cross_section,
                self.objects[i].color,
                &self.camera2d,
                &self.geng,
                &mut flat_buffer,
            );
        }
        draw_texture_to(&self.flat_texture, flat_pos, &self.geng, framebuffer);

        // UI
        let camera = &geng::PixelPerfectCamera;
        if include_3d {
            let color_normal = Rgba::try_from("#222").unwrap();
            let color_hover = Rgba::try_from("#555").unwrap();
            let color_drag = Rgba::try_from("#333").unwrap();

            // Horizontal
            let color = if let Some(Drag::Y) = self.drag {
                color_drag
            } else if self.separator_y.contains(self.cursor_pos) {
                color_hover
            } else {
                color_normal
            };
            self.separator_y = Aabb2::point(vec2(0.0, self.lower_left_size.y) * framebuffer_size)
                .extend_symmetric(vec2(0.0, 5.0))
                .extend_right(self.lower_left_size.x * framebuffer_size.x);
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(self.separator_y, color),
            );

            // Vertical
            let color = if let Some(Drag::X) = self.drag {
                color_drag
            } else if self.separator_x.contains(self.cursor_pos) {
                color_hover
            } else {
                color_normal
            };
            self.separator_x = Aabb2::point(vec2(self.lower_left_size.x, 0.0) * framebuffer_size)
                .extend_symmetric(vec2(5.0, 0.0))
                .extend_up(framebuffer_size.y);
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(self.separator_x, color),
            );
        }
    }
}

fn draw_flat_section(
    cross_section: &[PlaneSectionVertex],
    color: Rgba<f32>,
    camera: &Camera2d,
    geng: &Geng,
    framebuffer: &mut ugli::Framebuffer,
) {
    if cross_section.len() < 3 {
        return;
    }

    let mirror_x = |v: vec2<f32>| vec2(-v.x, v.y);
    let mut chain: Vec<vec2<f32>> = cross_section
        .iter()
        .map(|v| mirror_x(v.projected))
        .collect();
    let mid = (chain[0] + chain[1]) / 2.0;
    chain.extend([chain[0], mid]);
    chain[0] = mid;
    geng.draw2d().draw2d(
        framebuffer,
        camera,
        &draw2d::Chain::new(Chain::new(chain), 0.1, color, 5),
    );
}

fn attach<'a>(
    texture: &'a mut ugli::Texture,
    depth: &'a mut ugli::Renderbuffer<ugli::DepthComponent>,
    ugli: &Ugli,
) -> ugli::Framebuffer<'a> {
    ugli::Framebuffer::new(
        ugli,
        ugli::ColorAttachment::Texture(texture),
        ugli::DepthAttachment::Renderbuffer(depth),
    )
}

fn attach_clear<'a>(
    texture: &'a mut ugli::Texture,
    depth: &'a mut ugli::Renderbuffer<ugli::DepthComponent>,
    clear_color: Rgba<f32>,
    ugli: &Ugli,
) -> ugli::Framebuffer<'a> {
    let mut framebuffer = attach(texture, depth, ugli);
    ugli::clear(&mut framebuffer, Some(clear_color), Some(1.0), None);
    framebuffer
}

fn draw_texture_to(
    texture: &ugli::Texture,
    target: Aabb2<f32>,
    geng: &Geng,
    framebuffer: &mut ugli::Framebuffer,
) {
    texture_utils::draw_texture_fit(
        texture,
        target,
        vec2(0.5, 0.5),
        &geng::PixelPerfectCamera,
        geng,
        framebuffer,
    );
    // geng.draw2d().draw2d(
    //     framebuffer,
    //     &geng::PixelPerfectCamera,
    //     &draw2d::TexturedQuad::new(target, texture),
    // );
}
