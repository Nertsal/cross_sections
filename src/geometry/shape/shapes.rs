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

pub fn unit_tetrahedron_triangulized() -> Vec<Vertex> {
    let tri = |vs: [(f32, f32, f32); 3]| Triangle::new(vs.map(|(x, y, z)| vec3(x, y, z)));
    let [a, b, c, d] = unit_tetrahedron();
    let triangles = [
        tri([a, c, b]),
        tri([a, b, d]),
        tri([a, c, d]),
        tri([b, c, d]),
    ];
    triangles
        .into_iter()
        .flat_map(Triangle::into_vertices)
        .collect()
}

pub fn unit_icosahedron() -> [(f32, f32, f32); 12] {
    const PHI: f32 = 1.618_034; // Golden ratio
    [
        (0.0, 1.0, PHI),   // a  0
        (0.0, -1.0, PHI),  // b  1
        (0.0, 1.0, -PHI),  // c  2
        (0.0, -1.0, -PHI), // d  3
        (1.0, PHI, 0.0),   // e  4
        (-1.0, PHI, 0.0),  // f  5
        (1.0, -PHI, 0.0),  // g  6
        (-1.0, -PHI, 0.0), // h  7
        (PHI, 0.0, 1.0),   // i  8
        (-PHI, 0.0, 1.0),  // j  9
        (PHI, 0.0, -1.0),  // k 10
        (-PHI, 0.0, -1.0), // l 11
    ]
}

pub fn unit_icosahedron_triangulized() -> Vec<Vertex> {
    let tri = |vs: [(f32, f32, f32); 3]| Triangle::new(vs.map(|(x, y, z)| vec3(x, y, z)));
    let vs = unit_icosahedron();
    // a-jbief:  0 -  9  1  8  4  5
    // d-ghlck:  3 -  6  7 11  2 10
    // b-jhgi :  1 -  9  7  6  8
    // l-hjfc : 11 -  7  9  5  2
    // k-ceig : 10 -  2  4  8  6
    // ecf    :  4  2  5
    let triangles: [_; 20] = [
        (0, 9, 1),
        (0, 1, 8),
        (0, 8, 4),
        (0, 4, 5),
        (0, 5, 9),
        (3, 6, 7),
        (3, 7, 11),
        (3, 11, 2),
        (3, 2, 10),
        (3, 10, 6),
        (1, 9, 7),
        (1, 7, 6),
        (1, 6, 8),
        (11, 7, 9),
        (11, 9, 5),
        (11, 5, 2),
        (10, 2, 4),
        (10, 4, 8),
        (10, 8, 6),
        (4, 2, 5),
    ];
    triangles
        .into_iter()
        .map(|(a, b, c)| tri([vs[a], vs[b], vs[c]]))
        .flat_map(Triangle::into_vertices)
        .collect()
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
