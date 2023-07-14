use geng::prelude::*;

use super::{
    mat5,
    shape::{Tetrahedron4d, Triangle, Triangle4d},
    vec5,
};

#[derive(Debug, Clone)]
pub struct Space {
    pub offset: f32,
}

impl Space {
    pub fn matrix(&self) -> mat5<f32> {
        mat5::translate(vec4::UNIT_W * self.offset)
    }

    pub fn project(&self, point: vec4<f32>) -> vec4<f32> {
        point - vec4::UNIT_W * self.distance(point)
    }

    pub fn project3d(&self, point: vec4<f32>) -> vec3<f32> {
        let point = self.project(point);
        let point = (self.matrix() * vec5(point.x, point.y, point.z, point.w, 1.0)).into_4d();
        vec3(point.x, point.y, point.z)
    }

    pub fn distance(&self, point: vec4<f32>) -> f32 {
        vec4::dot(vec4::UNIT_W, point) - self.offset
    }

    pub fn intersect_segment(&self, p1: vec4<f32>, p2: vec4<f32>) -> Option<vec4<f32>> {
        let d1 = self.distance(p1);
        let d2 = self.distance(p2);

        if (d1 - d2).abs() < 1e-5 {
            // Parallel
            return None;
        }

        let t = d1 / (d1 - d2);
        (0.0..=1.0).contains(&t).then_some(p1 + t * (p2 - p1))
    }

    pub fn intersect_tetrahedron(&self, tetrahedron: &Tetrahedron4d) -> Vec<Triangle4d> {
        let points: Vec<vec4<f32>> = tetrahedron
            .edges()
            .into_iter()
            .flat_map(|(p1, p2)| self.intersect_segment(p1, p2))
            .collect();
        match points[..] {
            [a, b, c] => vec![Triangle4d {
                vertices: [a, b, c],
            }],
            [a, b, c, d] => vec![
                Triangle4d {
                    vertices: [a, b, c],
                },
                Triangle4d {
                    vertices: [d, c, b],
                },
            ],
            _ => vec![],
        }
    }

    /// Calculate a cross section of `geometry` with the space.
    pub fn cross_sect(
        &self,
        geometry: impl IntoIterator<Item = impl std::borrow::Borrow<Tetrahedron4d>>,
    ) -> Vec<Triangle> {
        let mut triangles: Vec<Triangle> = geometry
            .into_iter()
            .flat_map(|tetrahedron| {
                self.intersect_tetrahedron(tetrahedron.borrow())
                    .into_iter()
                    .map(|triangle| Triangle::new(triangle.vertices.map(|v| self.project3d(v))))
            })
            .collect();
        let center = triangles
            .iter()
            .flat_map(|t| t.vertices)
            .fold(vec3::ZERO, vec3::add)
            / (triangles.len() as f32 * 3.0);
        for triangle in &mut triangles {
            triangle.look_away_from(center);
        }
        triangles
    }
}
