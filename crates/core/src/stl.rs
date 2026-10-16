//! Bounded STL import. Exact vertex welding only; invalid surfaces are rejected, not repaired.
use crate::{document::*, geometry::Mesh};
use anyhow::{Context, Result, bail, ensure};
use glam::DVec3;
use manifold_rust::{
    manifold::Manifold,
    types::{Error, MeshGL64},
};
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    path::Path,
};
pub const MAX_TRIANGLES: usize = 20_000;
pub const MAX_STL_BYTES: usize = 10_000_000;

pub fn manifold(vertices: &[Vec3], triangles: &[[usize; 3]]) -> Manifold {
    Manifold::from_mesh_gl64(&MeshGL64 {
        num_prop: 3,
        vert_properties: vertices.iter().flatten().copied().collect(),
        tri_verts: triangles.iter().flatten().map(|v| *v as u64).collect(),
        ..Default::default()
    })
}

pub fn validate_mesh(vertices: &[Vec3], triangles: &[[usize; 3]]) -> Result<()> {
    ensure!(
        !triangles.is_empty() && triangles.len() <= MAX_TRIANGLES,
        "Mesh must contain 1–20,000 triangles"
    );
    ensure!(
        vertices.len() >= 4 && vertices.len() <= 3 * MAX_TRIANGLES,
        "Invalid mesh vertex count"
    );
    ensure!(
        vertices.iter().flatten().all(|v| v.is_finite()),
        "Mesh contains nonfinite coordinates"
    );
    let mut edges: HashMap<(usize, usize), (u32, i32)> = HashMap::new();
    let mut faces = HashSet::new();
    let mut used = HashSet::new();
    for t in triangles {
        ensure!(
            t.iter().all(|i| *i < vertices.len()),
            "Mesh index outside vertex array"
        );
        let mut face = *t;
        face.sort_unstable();
        ensure!(faces.insert(face), "Duplicate triangle");
        let [a, b, c] = t.map(|i| DVec3::from(vertices[i]));
        ensure!(
            (b - a).cross(c - a).length_squared() > 1e-28,
            "Degenerate triangle"
        );
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            used.insert(a);
            let entry = edges.entry((a.min(b), a.max(b))).or_default();
            entry.0 += 1;
            entry.1 += if a < b { 1 } else { -1 };
        }
    }
    ensure!(used.len() == vertices.len(), "Unused mesh vertices");
    ensure!(
        edges.values().all(|(n, d)| *n == 2 && *d == 0),
        "Mesh must be closed with consistent face winding (open/non-manifold edge found)"
    );
    Ok(())
}

pub fn validate_project_meshes(p: &Project) -> Result<()> {
    fn walk(n: &Node) -> Result<()> {
        match &n.shape {
            Shape::Mesh { triangles, .. } => {
                let m = manifold(&crate::geometry::points(n), triangles);
                ensure!(
                    m.status() == Error::NoError && !m.is_empty(),
                    "Mesh rejected by solid engine: {:?}",
                    m.status()
                );
                ensure!(
                    !m.has_self_intersections(),
                    "Self-intersecting mesh is not supported"
                );
                ensure!(
                    crate::geometry::mesh(&m).volume() > 1e-12,
                    "Mesh must have positive solid volume"
                );
            }
            Shape::Group { children, .. } => {
                for child in children {
                    walk(child)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    for n in &p.parts {
        walk(n)?;
    }
    Ok(())
}

pub fn read(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((MAX_STL_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= MAX_STL_BYTES, "STL exceeds 10 MB");
    Ok(bytes)
}

pub fn import(bytes: &[u8], name: &str, millimeters_per_unit: f64) -> Result<Node> {
    ensure!(bytes.len() <= MAX_STL_BYTES, "STL exceeds 10 MB");
    ensure!(
        millimeters_per_unit.is_finite() && millimeters_per_unit > 0.,
        "Invalid import scale"
    );
    let mut soup: Vec<[Vec3; 3]> = Vec::new();
    let binary_count = if bytes.len() >= 84 {
        Some(u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize)
    } else {
        None
    };
    if let Some(count) =
        binary_count.filter(|n| 84usize.checked_add(n.saturating_mul(50)) == Some(bytes.len()))
    {
        ensure!(
            count > 0 && count <= MAX_TRIANGLES,
            "STL must contain 1–20,000 triangles"
        );
        for record in bytes[84..].as_chunks::<50>().0 {
            let mut tri = [[0.; 3]; 3];
            for (i, vertex) in tri.iter_mut().enumerate() {
                for (j, value) in vertex.iter_mut().enumerate() {
                    let offset = 12 + i * 12 + j * 4;
                    *value =
                        f32::from_le_bytes(record[offset..offset + 4].try_into().unwrap()) as f64;
                }
            }
            soup.push(tri);
        }
    } else {
        let ascii = std::str::from_utf8(bytes).context("Truncated or invalid binary STL")?;
        let mut lines = ascii.lines().map(str::trim).filter(|s| !s.is_empty());
        ensure!(
            lines
                .next()
                .is_some_and(|l| l.split_whitespace().next() == Some("solid")),
            "Expected ASCII solid or complete binary STL"
        );
        let mut ended = false;
        while let Some(line) = lines.next() {
            if line.split_whitespace().next() == Some("endsolid") {
                ended = true;
                break;
            }
            let words: Vec<_> = line.split_whitespace().collect();
            ensure!(
                words.len() == 5 && words[..2] == ["facet", "normal"],
                "Expected facet normal"
            );
            for word in &words[2..] {
                ensure!(word.parse::<f64>()?.is_finite(), "Invalid normal");
            }
            ensure!(lines.next() == Some("outer loop"), "Expected outer loop");
            let mut tri = [[0.; 3]; 3];
            for vertex in &mut tri {
                let words: Vec<_> = lines
                    .next()
                    .context("Truncated triangle")?
                    .split_whitespace()
                    .collect();
                ensure!(
                    words.len() == 4 && words[0] == "vertex",
                    "Expected vertex x y z"
                );
                for i in 0..3 {
                    vertex[i] = words[i + 1].parse()?;
                }
            }
            ensure!(
                lines.next() == Some("endloop") && lines.next() == Some("endfacet"),
                "Incomplete facet"
            );
            soup.push(tri);
            ensure!(soup.len() <= MAX_TRIANGLES, "STL exceeds 20,000 triangles");
        }
        ensure!(
            ended && lines.next().is_none(),
            "Missing endsolid or unsupported trailing data/multiple solids"
        );
    }
    if soup.is_empty() {
        bail!("Empty STL");
    }
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut triangles = Vec::new();
    let mut indices = HashMap::new();
    for tri in soup {
        let mut face = [0; 3];
        for (i, mut v) in tri.into_iter().enumerate() {
            for c in &mut v {
                *c *= millimeters_per_unit;
                ensure!(
                    c.is_finite() && c.abs() <= 1e9,
                    "Invalid or excessively large STL coordinates"
                );
                if *c == 0. {
                    *c = 0.;
                } // weld signed zero
            }
            face[i] = *indices.entry(v.map(f64::to_bits)).or_insert_with(|| {
                let index = vertices.len();
                vertices.push(v);
                index
            });
        }
        triangles.push(face);
    }
    validate_mesh(&vertices, &triangles)?;
    let min = vertices
        .iter()
        .fold(DVec3::splat(f64::INFINITY), |a, v| a.min(DVec3::from(*v)));
    let max = vertices
        .iter()
        .fold(DVec3::splat(f64::NEG_INFINITY), |a, v| {
            a.max(DVec3::from(*v))
        });
    let center = (min + max) / 2.;
    let size = (max - min).to_array();
    ensure!(
        size.iter().all(|v| *v >= 0.01 && *v <= 1000.),
        "Imported dimensions must be 0.01–1000 mm; choose appropriate units"
    );
    // Recenter before volume calculation to avoid cancellation for offset STLs.
    for v in &mut vertices {
        *v = (DVec3::from(*v) - center).to_array();
    }
    let volume = Mesh {
        vertices: vertices.clone(),
        triangles: triangles.clone(),
    }
    .volume();
    ensure!(volume.abs() > 1e-12, "STL has zero solid volume");
    if volume < 0. {
        for t in &mut triangles {
            t.swap(1, 2);
        }
    }
    for v in &mut vertices {
        for axis in 0..3 {
            v[axis] /= size[axis];
        }
    }
    let clean_name: String = name.chars().filter(|c| *c >= ' ').take(100).collect();
    let node = Node {
        id: uuid::Uuid::new_v4().to_string(),
        name: if clean_name.trim().is_empty() {
            "Imported STL".into()
        } else {
            clean_name
        },
        hole: false,
        hidden: false,
        locked: false,
        position: [40., 15., size[2] / 2.],
        rotation: [0.; 3],
        shape: Shape::Mesh {
            size,
            vertices,
            triangles,
        },
    };
    let p = Project {
        parts: vec![node.clone()],
        ..Project::default()
    };
    p.validate()?;
    validate_project_meshes(&p)?;
    Ok(node)
}
