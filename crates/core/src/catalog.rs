//! Procedural shapes and an original 5×7 block alphabet; no system font dependency.
use crate::{Node, Shape, document::Vec3};
use anyhow::{Result, ensure};
use manifold_rust::{linalg::Vec3 as V, manifold::Manifold};
const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._+ ";
// Rows are five-bit masks, top to bottom. This deliberately simple alphabet is
// generated as overlapping voxel runs, giving diagonal strokes real connections.
const GLYPHS: [[u8; 7]; 40] = [
    [14, 17, 17, 31, 17, 17, 17],
    [30, 17, 17, 30, 17, 17, 30],
    [14, 17, 16, 16, 16, 17, 14],
    [30, 17, 17, 17, 17, 17, 30],
    [31, 16, 16, 30, 16, 16, 31],
    [31, 16, 16, 30, 16, 16, 16],
    [14, 17, 16, 23, 17, 17, 14],
    [17, 17, 17, 31, 17, 17, 17],
    [31, 4, 4, 4, 4, 4, 31],
    [7, 2, 2, 2, 18, 18, 12],
    [17, 18, 20, 24, 20, 18, 17],
    [16, 16, 16, 16, 16, 16, 31],
    [17, 27, 21, 21, 17, 17, 17],
    [17, 25, 21, 19, 17, 17, 17],
    [14, 17, 17, 17, 17, 17, 14],
    [30, 17, 17, 30, 16, 16, 16],
    [14, 17, 17, 17, 21, 18, 13],
    [30, 17, 17, 30, 20, 18, 17],
    [15, 16, 16, 14, 1, 1, 30],
    [31, 4, 4, 4, 4, 4, 4],
    [17, 17, 17, 17, 17, 17, 14],
    [17, 17, 17, 17, 17, 10, 4],
    [17, 17, 17, 21, 21, 21, 10],
    [17, 17, 10, 4, 10, 17, 17],
    [17, 17, 10, 4, 4, 4, 4],
    [31, 1, 2, 4, 8, 16, 31],
    [14, 17, 19, 21, 25, 17, 14],
    [4, 12, 4, 4, 4, 4, 14],
    [14, 17, 1, 2, 4, 8, 31],
    [30, 1, 1, 14, 1, 1, 30],
    [2, 6, 10, 18, 31, 2, 2],
    [31, 16, 16, 30, 1, 1, 30],
    [14, 16, 16, 30, 17, 17, 14],
    [31, 1, 2, 4, 8, 8, 8],
    [14, 17, 17, 14, 17, 17, 14],
    [14, 17, 17, 15, 1, 1, 14],
    [0, 0, 0, 31, 0, 0, 0],
    [0, 0, 0, 0, 0, 6, 6],
    [0, 0, 0, 0, 0, 0, 31],
    [0, 4, 4, 31, 4, 4, 0],
];
pub fn validate_text(text: &str) -> Result<()> {
    ensure!(
        !text.trim().is_empty() && text.len() <= 24,
        "Use 1–24 text characters"
    );
    ensure!(
        text.chars()
            .all(|c| c.is_ascii() && ALPHABET.contains(c.to_ascii_uppercase())),
        "Block text supports A–Z, 0–9, spaces and - . _ +"
    );
    Ok(())
}
fn text_boxes(content: &str, size: Vec3, mirrored: [bool; 3]) -> Vec<([f64; 3], [f64; 3])> {
    let mut boxes = vec![];
    for (col, c) in content.chars().enumerate() {
        let Some(g) = ALPHABET
            .find(c.to_ascii_uppercase())
            .filter(|&i| i < 40)
            .map(|i| GLYPHS[i])
        else {
            continue;
        };
        for (row, bits) in g.iter().enumerate() {
            let mut x = 0;
            while x < 5 {
                if bits & (1 << (4 - x)) == 0 {
                    x += 1;
                    continue;
                }
                let start = x;
                while x < 5 && bits & (1 << (4 - x)) != 0 {
                    x += 1;
                }
                boxes.push((
                    [
                        col as f64 * 6. + start as f64 - 0.02,
                        6. - row as f64 - 0.02,
                        -0.5,
                    ],
                    [
                        col as f64 * 6. + x as f64 + 0.02,
                        7. - row as f64 + 0.02,
                        0.5,
                    ],
                ));
            }
        }
    }
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for (a, b) in &boxes {
        for k in 0..3 {
            min[k] = min[k].min(a[k]);
            max[k] = max[k].max(b[k]);
        }
    }
    for (a, b) in &mut boxes {
        for k in 0..3 {
            a[k] = ((a[k] - min[k]) / (max[k] - min[k]) - 0.5) * size[k];
            b[k] = ((b[k] - min[k]) / (max[k] - min[k]) - 0.5) * size[k];
            if mirrored[k] {
                (a[k], b[k]) = (-b[k], -a[k]);
            }
        }
    }
    boxes
}
fn torus(n: &Node) -> (Vec<Vec3>, Vec<[usize; 3]>) {
    let Shape::Torus { size, bore } = &n.shape else {
        unreachable!()
    };
    let around = n.segments as usize;
    let ring = (n.segments / 4).div_ceil(4) as usize * 4;
    let major = size[0] * (1. + bore) / 4.;
    let minor = size[0] * (1. - bore) / 4.;
    let mut vertices = vec![];
    let mut triangles = vec![];
    for i in 0..around {
        let a = i as f64 * std::f64::consts::TAU / around as f64;
        for j in 0..ring {
            let b = j as f64 * std::f64::consts::TAU / ring as f64;
            let r = major + minor * b.cos();
            vertices.push([r * a.cos(), r * a.sin(), size[2] / 2. * b.sin()]);
        }
    }
    for i in 0..around {
        for j in 0..ring {
            let a = i * ring + j;
            let b = ((i + 1) % around) * ring + j;
            let c = ((i + 1) % around) * ring + (j + 1) % ring;
            let d = i * ring + (j + 1) % ring;
            triangles.push([a, b, c]);
            triangles.push([a, c, d]);
        }
    }
    (vertices, triangles)
}
pub fn points(n: &Node) -> Vec<Vec3> {
    match &n.shape {
        Shape::Torus { .. } => torus(n).0,
        Shape::Text {
            size,
            content,
            mirrored,
        } => text_boxes(content, *size, *mirrored)
            .iter()
            .flat_map(|(a, b)| {
                let mut v = vec![];
                for x in [a[0], b[0]] {
                    for y in [a[1], b[1]] {
                        for z in [a[2], b[2]] {
                            v.push([x, y, z]);
                        }
                    }
                }
                v
            })
            .collect(),
        Shape::Tube { size, .. } | Shape::Polygon { size, .. } => {
            let segments = if let Shape::Polygon { sides, .. } = n.shape {
                sides
            } else {
                n.segments
            };
            let mut p = vec![];
            for i in 0..segments {
                let a = i as f64 * std::f64::consts::TAU / segments as f64;
                for z in [-size[2] / 2., size[2] / 2.] {
                    p.push([size[0] / 2. * a.cos(), size[1] / 2. * a.sin(), z]);
                }
            }
            p
        }
        _ => unreachable!(),
    }
}
pub fn primitive(n: &Node) -> Manifold {
    match &n.shape {
        Shape::Tube { size, bore } => Manifold::cylinder_centered(
            size[2],
            size[0] / 2.,
            size[0] / 2.,
            n.segments as i32,
            true,
        )
        .difference(&Manifold::cylinder_centered(
            size[2] + 2.,
            size[0] * bore / 2.,
            size[0] * bore / 2.,
            n.segments as i32,
            true,
        )),
        Shape::Polygon { .. } => Manifold::hull(
            &points(n)
                .iter()
                .map(|v| V::new(v[0], v[1], v[2]))
                .collect::<Vec<_>>(),
        ),
        Shape::Torus { .. } => {
            let (v, t) = torus(n);
            crate::stl::manifold(&v, &t)
        }
        Shape::Text {
            size,
            content,
            mirrored,
        } => {
            let mut result = Manifold::empty();
            for (a, b) in text_boxes(content, *size, *mirrored) {
                let cube =
                    Manifold::cube(V::new(b[0] - a[0], b[1] - a[1], b[2] - a[2]), true).translate(
                        V::new((a[0] + b[0]) / 2., (a[1] + b[1]) / 2., (a[2] + b[2]) / 2.),
                    );
                result = result.union(&cube);
            }
            result
        }
        _ => unreachable!(),
    }
}
