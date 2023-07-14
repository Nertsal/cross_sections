use crate::{
    geometry::{
        plane::{Plane, PlaneSectionVertex},
        shape::Triangle,
        Vertex,
    },
    Assets,
};

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

struct Object {
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

pub struct State2d {
    geng: Geng,
    assets: Rc<Assets>,
    framebuffer_size: vec2<usize>,
    camera: Camera2d,
    simulation_time: f32,
    next_spawn: f32,
    prefabs: Vec<Rc<ugli::VertexBuffer<Vertex>>>,
    objects: Vec<Object>,
}

impl State2d {
    pub fn new(geng: Geng, assets: Rc<Assets>) -> Self {
        let prefab = |geometry| Rc::new(ugli::VertexBuffer::new_dynamic(geng.ugli(), geometry));
        Self {
            framebuffer_size: vec2(1, 1),
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            simulation_time: 0.0,
            next_spawn: 0.0,
            prefabs: vec![prefab(crate::geometry::shape::unit_cube_triangulated())],
            objects: Vec::new(),
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
            .camera
            .projection_matrix(self.framebuffer_size.as_f32())
            * self.camera.view_matrix())
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

    pub fn draw(&mut self, include_3d: bool, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(
            framebuffer,
            Some(self.assets.config.get().background_color),
            None,
            None,
        );

        let cross_plane = Plane {
            normal: vec3(0.0, 0.0, 1.0),
            offset: 0.0,
        };

        // Calculate a cross section
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

        // Draw the cross section in 2d
        for (i, cross_section) in &cross_sections {
            let i = *i;
            draw_flat_section(
                cross_section,
                self.objects[i].color,
                &self.camera,
                &self.geng,
                framebuffer,
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
