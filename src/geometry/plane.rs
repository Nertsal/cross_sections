use super::shape::Triangle;

use geng::prelude::*;

#[derive(Debug, Clone)]
pub struct Plane {
    pub normal: vec3<f32>,
    pub offset: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct PlaneSectionVertex {
    pub world_pos: vec3<f32>,
    pub projected: vec2<f32>,
}

impl Plane {
    pub fn matrix(&self) -> mat4<f32> {
        let flat = vec2(self.normal.x, self.normal.z);
        let rot_h = flat.arg();
        let rot_v = vec2(flat.len(), self.normal.y).arg();
        mat4::rotate_z(-rot_v)
            * mat4::rotate_y(rot_h)
            * mat4::translate(-self.normal.normalize_or_zero() * self.offset)
    }

    pub fn project(&self, point: vec3<f32>) -> vec3<f32> {
        point - self.normal.normalize_or_zero() * self.distance(point)
    }

    pub fn project2d(&self, point: vec3<f32>) -> vec2<f32> {
        let point = self.project(point);
        let point = (self.matrix() * point.extend(1.0)).into_3d();
        vec2(point.z, point.y)
    }

    pub fn distance(&self, point: vec3<f32>) -> f32 {
        vec3::dot(self.normal.normalize_or_zero(), point) - self.offset
    }

    pub fn intersect_segment(&self, p1: vec3<f32>, p2: vec3<f32>) -> Option<vec3<f32>> {
        let d1 = self.distance(p1);
        let d2 = self.distance(p2);

        if (d1 - d2).abs() < 1e-5 {
            // Parallel
            return None;
        }

        let t = d1 / (d1 - d2);
        (0.0..=1.0).contains(&t).then_some(p1 + t * (p2 - p1))
    }

    pub fn intersect_triangle(&self, triangle: &Triangle) -> Option<(vec3<f32>, vec3<f32>)> {
        let [a, b, c] = triangle.vertices;
        let points: Vec<vec3<f32>> = [(a, b), (a, c), (b, c)]
            .into_iter()
            .flat_map(|(p1, p2)| self.intersect_segment(p1, p2))
            .collect();
        match &points[..] {
            [a, b] => Some((*a, *b)),
            _ => None,
        }
    }

    /// Calculate a cross section of `geometry` with the plane.
    pub fn cross_sect(
        &self,
        geometry: impl IntoIterator<Item = impl std::borrow::Borrow<Triangle>>,
    ) -> Vec<PlaneSectionVertex> {
        let mut points: Vec<PlaneSectionVertex> = Vec::new();
        for triangle in geometry {
            if let Some((a, b)) = self.intersect_triangle(triangle.borrow()) {
                for p in [a, b] {
                    let mut found = false;
                    for q in &points {
                        if (q.world_pos - p).len_sqr() < 1e-5 {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        points.push(PlaneSectionVertex {
                            world_pos: p,
                            projected: self.project2d(p),
                        });
                    }
                }
            }
        }

        if !points.is_empty() {
            // Sort counter clockwise
            let com = points
                .iter()
                .map(|p| p.projected)
                .fold(vec2::ZERO, vec2::add)
                / points.len() as f32;
            points.sort_by_key(|p| -r32((p.projected - com).arg().as_radians()));
        }

        points
    }
}

#[test]
fn test_plane_project() {
    macro_rules! check {
        ($a:expr, $b:expr) => {{
            let a = $a;
            let b = $b;
            let delta = (b - a).len();
            assert!(
                delta < 1e-5,
                "\n  left: `{:?}`,\n right: `{:?}`,\n delta: `{}`",
                a,
                b,
                delta
            );
        }};
    }

    let offsets = [0.0, 2.0, -3.0];

    for offset in offsets {
        println!("Testing offset {}", offset);

        let plane = Plane {
            normal: vec3::UNIT_X,
            offset,
        };

        check!(plane.project(vec3(10.0, 2.0, 1.0)), vec3(offset, 2.0, 1.0));
        check!(plane.project2d(vec3(10.0, 2.0, 1.0)), vec2(1.0, 2.0));

        let plane = Plane {
            normal: vec3::UNIT_Z,
            offset,
        };
        check!(plane.project(vec3(1.0, 2.0, 10.0)), vec3(1.0, 2.0, offset));
        check!(plane.project2d(vec3(1.0, 2.0, 10.0)), vec2(-1.0, 2.0));

        let plane = Plane {
            normal: vec3(1.0, 1.0, 0.0),
            offset,
        };
        check!(
            plane.project(vec3(1.0, 1.0, 2.0)),
            vec3(offset / 2.0.sqrt(), offset / 2.0.sqrt(), 2.0)
        );
        check!(plane.project2d(vec3(1.0, 1.0, 2.0)), vec2(2.0, 0.0));
    }
}
