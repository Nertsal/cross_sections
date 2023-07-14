mod shapes;

pub use self::shapes::*;

use super::Vertex;

use geng::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub vertices: [vec3<f32>; 3],
    pub normal: vec3<f32>,
}

impl Triangle {
    pub fn new([a, b, c]: [vec3<f32>; 3]) -> Self {
        let ab = b - a;
        let ac = c - a;
        let normal = vec3(
            ab.y * ac.z - ab.z * ac.y,
            ab.z * ac.x - ab.x * ac.z,
            ab.x * ac.y - ab.y * ac.x,
        )
        .normalize_or_zero();
        Self {
            vertices: [a, b, c],
            normal,
        }
    }

    pub fn into_vertices(self) -> [Vertex; 3] {
        self.vertices.map(|a_pos| Vertex::white(a_pos, self.normal))
    }

    pub fn center(&self) -> vec3<f32> {
        self.vertices.into_iter().fold(vec3::ZERO, vec3::add) / 3.0
    }

    /// Changes the normal to be looking in the opposite direction of the `target`
    /// without changing the vertices' positions.
    /// So it may or may not flip the normal.
    pub fn look_away_from(&mut self, target: vec3<f32>) {
        let center = self.center();
        if vec3::dot(self.normal, target - center) > 0.0 {
            // Flip the normal
            self.normal = -self.normal;
            self.vertices.swap(0, 1);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle4d {
    pub vertices: [vec4<f32>; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct Tetrahedron4d {
    pub vertices: [vec4<f32>; 4],
}

impl Tetrahedron4d {
    pub fn edges(&self) -> [(vec4<f32>, vec4<f32>); 6] {
        let [a, b, c, d] = self.vertices;
        [(a, b), (a, c), (a, d), (b, c), (b, d), (c, d)]
    }
}
