use geng::prelude::*;

use super::*;

/// `M` must be equal to `N * 2`
fn array_flatten<T: Copy, const N: usize, const M: usize>(arr: [[T; N]; 2]) -> [T; M] {
    arr.into_iter()
        .flatten()
        .collect::<Vec<T>>()
        .as_slice()
        .try_into()
        .unwrap()
}

pub fn unit_line() -> [f32; 2] {
    [-1.0, 1.0]
}

pub fn unit_square() -> [(f32, f32); 4] {
    array_flatten([
        unit_line().map(|x| (x, -1.0)),
        unit_line().map(|x| (x, 1.0)),
    ])
}

pub fn unit_cube() -> [(f32, f32, f32); 8] {
    array_flatten([
        unit_square().map(|(x, y)| (x, y, -1.0)),
        unit_square().map(|(x, y)| (x, y, 1.0)),
    ])
}

pub fn unit_cube_triangulated() -> Vec<Vertex> {
    let vertices = unit_cube().map(|(x, y, z)| vec3(x, y, z));

    let triangles = [
        [0, 3, 1],
        [0, 2, 3],
        [0, 5, 4],
        [0, 1, 5],
        [1, 7, 5],
        [1, 3, 7],
        [2, 4, 6],
        [2, 0, 4],
        [3, 6, 7],
        [3, 2, 6],
        [4, 7, 6],
        [4, 5, 7],
    ];
    triangles
        .into_iter()
        .flat_map(|ids| Triangle::new(ids.map(|i| vertices[i])).into_vertices())
        .collect()
}

pub fn unit_triangle() -> [(f32, f32); 3] {
    const HEIGHT: f32 = 1.73205;
    [
        (-1.0, -HEIGHT / 3.0),
        (1.0, -HEIGHT / 3.0),
        (0.0, HEIGHT * 2.0 / 3.0),
    ]
}

pub fn unit_tetrahedron() -> [(f32, f32, f32); 4] {
    // const HEIGHT: f32 = 1.73205;
    // let [a, b, c] = unit_triangle().map(|(x, z)| (x, -HEIGHT / 3.0, z));
    // [a, b, c, (0.0, HEIGHT * 2.0 / 3.0, 0.0)]
    [
        (1.0, 1.0, 1.0),
        (1.0, -1.0, -1.0),
        (-1.0, 1.0, -1.0),
        (-1.0, -1.0, 1.0),
    ]
}

pub fn unit_5cell() -> [(f32, f32, f32, f32); 5] {
    // const HEIGHT: f32 = 1.73205;
    // let [a, b, c, d] = unit_tetrahedron().map(|(x, y, z)| (x, y, z, -HEIGHT / 3.0));
    // [a, b, c, d, (0.0, 0.0, 0.0, HEIGHT * 2.0 / 3.0)]
    let root_five = 5.0.sqrt();
    let [a, b, c, d] = unit_tetrahedron().map(|(x, y, z)| (x, y, z, -1.0 / root_five));
    [a, b, c, d, (0.0, 0.0, 0.0, 4.0 / root_five)]
}

pub fn unit_5cell_tetrahedralized() -> [Tetrahedron4d; 5] {
    let tetra = |vs: [(f32, f32, f32, f32); 4]| Tetrahedron4d {
        vertices: vs.map(|(x, y, z, w)| vec4(x, y, z, w)),
    };
    let [a, b, c, d, e] = unit_5cell();
    [
        tetra([a, b, c, d]),
        tetra([a, b, c, e]),
        tetra([a, b, d, e]),
        tetra([a, c, d, e]),
        tetra([b, c, d, e]),
    ]
}
