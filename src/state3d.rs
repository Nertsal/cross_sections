use crate::{
    camera3d::Camera3d,
    geometry::{mat5, shape::Tetrahedron4d, space::Space, vec4_len, vec4_norm, vec5, Vertex},
    Assets, Config,
};

use geng::prelude::*;
use geng_utils::{conversions::Vec2RealConversions, texture as texture_utils};

pub struct Object {
    pub geometry: Rc<[Tetrahedron4d]>,
    pub position: vec4<f32>,
    pub orientation: vec4<f32>,
    pub roll: Angle<f32>,
    pub scale: f32,
    pub color: Rgba<f32>,
}

impl Object {
    pub fn new(position: vec4<f32>, geometry: Rc<[Tetrahedron4d]>) -> Self {
        Self {
            geometry,
            position,
            orientation: vec4::UNIT_X,
            roll: Angle::ZERO,
            scale: 1.0,
            color: Rgba::WHITE,
        }
    }

    pub fn matrix(&self) -> mat5<f32> {
        let flat = vec2(self.orientation.x, self.orientation.z);
        let rot_xy = flat.arg();
        let vert = vec2(flat.len(), self.orientation.y);
        let rot_xz = vert.arg();
        let rot_yw = vec2(vert.len(), self.orientation.w).arg();
        mat5::translate(self.position)
            * mat5::rotate_zw(self.roll)
            * mat5::rotate_xz(rot_xz)
            * mat5::rotate_xy(rot_xy)
            * mat5::rotate_yw(rot_yw)
            * mat5::scale_uniform(self.scale)
    }

    pub fn rotate_zw(&mut self, angle: Angle<f32>) {
        let flat = vec2(self.orientation.z, self.orientation.w);
        let flat = flat.rotate(angle);
        self.orientation.z = flat.x;
        self.orientation.w = flat.y;
    }
}

pub struct State3d {
    geng: Geng,
    assets: Rc<Assets>,
    unit_geometry: Rc<ugli::VertexBuffer<draw2d::TexturedVertex>>,
    framebuffer_size: vec2<usize>,
    screen_texture: ugli::Texture,
    normal_texture: ugli::Texture,
    depth_buffer: ugli::Renderbuffer<ugli::DepthComponent>,
    simulation_time: f32,
    next_spawn: f32,
    prefabs: Vec<Rc<[Tetrahedron4d]>>,
    objects: Vec<Object>,
    camera: Camera3d,
    paused: bool,
}

impl State3d {
    pub fn new(geng: Geng, assets: Rc<Assets>) -> Self {
        let prefab = |geometry: &[_]| Rc::from(geometry);
        Self {
            unit_geometry: Rc::new(geng_utils::geometry::unit_quad_geometry(geng.ugli())),
            simulation_time: 0.0,
            next_spawn: 0.0,
            framebuffer_size: vec2(1, 1),
            screen_texture: texture_utils::new_texture(geng.ugli(), vec2(1, 1)),
            normal_texture: texture_utils::new_texture(geng.ugli(), vec2(1, 1)),
            depth_buffer: ugli::Renderbuffer::new(geng.ugli(), vec2(1, 1)),
            camera: Camera3d {
                fov: Angle::from_radians(70.0),
                pos: vec3(0.0, 0.0, 10.0),
                rot_h: Angle::ZERO,
                rot_v: Angle::ZERO,
                near: 1.0,
                far: 50.0,
            },
            objects: Vec::new(),
            prefabs: vec![prefab(&crate::geometry::shape::unit_5cell_tetrahedralized())],
            paused: false,
            geng,
            assets,
        }
    }

    pub fn random_spawn(&self, config: &Config) -> vec3<f32> {
        let mut rng = thread_rng();
        let x = rng.gen_range(-1.0..=1.0);
        let y = rng.gen_range(-1.0..=1.0);
        let z = rng.gen_range(config.spawn_depth_min..=config.spawn_depth_max);
        let pos = vec3(x, y, z);
        let pos = (self
            .camera
            .projection_matrix(self.framebuffer_size.as_f32())
            * self.camera.view_matrix())
        .inverse()
            * pos.extend(1.0);
        pos.into_3d()
    }

    pub fn handle_event(&mut self, event: &geng::Event) {
        if geng_utils::key::is_event_press(event, [geng::Key::P]) {
            self.paused = !self.paused;
        }
    }

    pub fn update(&mut self, config: &Config, delta_time: f64) {
        let delta_time = delta_time as f32;

        self.simulation_time += delta_time;
        self.next_spawn -= delta_time;
        let mut rng = thread_rng();
        while self.next_spawn < 0.0 {
            if self.objects.len() >= config.object_limit.value() {
                self.next_spawn = 1.0;
                break;
            }

            self.next_spawn += 0.1;
            if let Some(geometry) = self.prefabs.choose(&mut rng) {
                let scale = rng.gen_range(config.scale_min..=config.scale_max);
                let pos_w = -scale * 2.0;

                let pos = 'outer: {
                    let mut pos = self.random_spawn(config).extend(pos_w);
                    for _ in 0..5 {
                        let mut good = true;
                        for obj in &self.objects {
                            let dist = vec4_len(pos - obj.position);
                            if dist < (scale + obj.scale) * 2.0 {
                                // Try another one
                                pos = self.random_spawn(config).extend(pos_w);
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
                    obj.orientation = vec4_norm(vec4(
                        rng.gen_range(-1.0..=1.0),
                        rng.gen_range(-1.0..=1.0),
                        rng.gen_range(-1.0..=1.0),
                        rng.gen_range(-1.0..=1.0),
                    ));
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
            obj.position += vec4::UNIT_W * config.speed * delta_time;
            obj.rotate_zw(Angle::from_degrees(
                config.rotation_speed_degrees * delta_time,
            ));
        }
        // Delete far objects
        self.objects.retain(|obj| obj.position.w < obj.scale * 2.0);
    }

    pub fn draw(&mut self, config: &Config, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(config.background_color), Some(1.0), None);

        self.framebuffer_size = framebuffer.size();

        if self.screen_texture.size() != self.framebuffer_size {
            self.depth_buffer = ugli::Renderbuffer::new(self.geng.ugli(), self.framebuffer_size);
        }
        texture_utils::update_texture_size(
            &mut self.screen_texture,
            self.framebuffer_size,
            self.geng.ugli(),
        );
        texture_utils::update_texture_size(
            &mut self.normal_texture,
            self.framebuffer_size,
            self.geng.ugli(),
        );

        let cross_space = Space { offset: 0.0 };

        // Calculate a cross section
        let geometry: Vec<Vertex> = self
            .objects
            .iter()
            .flat_map(|obj| {
                let matrix = obj.matrix();
                let transform = |v: vec4<f32>| (matrix * vec5(v.x, v.y, v.z, v.w, 1.0)).into_4d();
                cross_space
                    .cross_sect(obj.geometry.iter().map(|tetra| Tetrahedron4d {
                        vertices: tetra.vertices.map(transform),
                    }))
                    .into_iter()
                    .flat_map(|triangle| triangle.into_vertices().map(|v| v.colored(obj.color)))
            })
            .collect();
        let geometry = ugli::VertexBuffer::new_dynamic(self.geng.ugli(), geometry);

        {
            // Draw the cross section in 2d
            let mut screen_buffer = ugli::Framebuffer::new(
                self.geng.ugli(),
                ugli::ColorAttachment::Texture(&mut self.screen_texture),
                ugli::DepthAttachment::Renderbuffer(&mut self.depth_buffer),
            );
            ugli::clear(
                &mut screen_buffer,
                Some(config.background_color),
                Some(1.0),
                None,
            );

            draw_with(
                &geometry,
                &self.camera,
                &self.assets.simple3d.get(),
                &mut screen_buffer,
            );
        }

        {
            // Mark the normals
            let mut normal_buffer = ugli::Framebuffer::new(
                self.geng.ugli(),
                ugli::ColorAttachment::Texture(&mut self.normal_texture),
                ugli::DepthAttachment::Renderbuffer(&mut self.depth_buffer),
            );
            ugli::clear(
                &mut normal_buffer,
                Some(config.background_color),
                Some(1.0),
                None,
            );

            draw_with(
                &geometry,
                &self.camera,
                &self.assets.outline_marker.get(),
                &mut normal_buffer,
            );
        }

        // Postprocess
        ugli::draw(
            framebuffer,
            &self.assets.postprocess.get(),
            ugli::DrawMode::TriangleFan,
            &*self.unit_geometry,
            ugli::uniforms! {
                u_color_texture: &self.screen_texture,
                u_outline_texture: &self.normal_texture,
                u_outline_texture_size: self.normal_texture.size(),
                u_outline_color: Rgba::BLACK,
            },
            ugli::DrawParameters { ..default() },
        );
    }
}

fn draw_with<T: ugli::Vertex>(
    geometry: &ugli::VertexBuffer<T>,
    camera: &Camera3d,
    program: &ugli::Program,
    framebuffer: &mut ugli::Framebuffer,
) {
    ugli::draw(
        framebuffer,
        program,
        ugli::DrawMode::Triangles,
        geometry,
        (
            ugli::uniforms! {
                u_model_matrix: mat4::identity(),
            },
            camera.uniforms(framebuffer.size().as_f32()),
        ),
        ugli::DrawParameters {
            cull_face: Some(ugli::CullFace::Back),
            depth_func: Some(ugli::DepthFunc::Less),
            ..default()
        },
    );
}
