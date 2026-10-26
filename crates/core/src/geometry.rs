use crate::document::*;
use anyhow::{Result, ensure};
use glam::{DMat4, DVec3};
use manifold_rust::{linalg::Vec3 as MVec, manifold::Manifold, types::Error};
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[usize; 3]>,
}
pub fn points(n: &Node) -> Vec<Vec3> {
    let Some(s) = n.size() else { return vec![] };
    let mut p = vec![];
    match &n.shape {
        Shape::Tube { .. } | Shape::Torus { .. } | Shape::Polygon { .. } | Shape::Text { .. } => {
            return crate::catalog::points(n);
        }
        Shape::Box { .. } => {
            for x in [-0.5, 0.5] {
                for y in [-0.5, 0.5] {
                    for z in [-0.5, 0.5] {
                        p.push([x * s[0], y * s[1], z * s[2]]);
                    }
                }
            }
        }
        Shape::Wedge { .. } | Shape::Roof { .. } => {
            for y in [-0.5, 0.5] {
                p.push([-s[0] / 2., y * s[1], -s[2] / 2.]);
                p.push([s[0] / 2., y * s[1], -s[2] / 2.]);
                p.push([
                    if matches!(n.shape, Shape::Wedge { .. }) {
                        -s[0] / 2.
                    } else {
                        0.
                    },
                    y * s[1],
                    s[2] / 2.,
                ]);
            }
        }
        Shape::Mesh { vertices, .. } => {
            p.extend(
                vertices
                    .iter()
                    .map(|v| [v[0] * s[0], v[1] * s[1], v[2] * s[2]]),
            );
        }
        Shape::Sphere { .. } => {
            // Default 96 retains the legacy 48-longitude / 24-latitude sphere.
            let latitudes = n.segments / 4;
            let longitudes = n.segments / 2;
            for lat in 0..=latitudes {
                let theta = lat as f64 * std::f64::consts::PI / latitudes as f64;
                for lon in 0..longitudes {
                    let phi = lon as f64 * std::f64::consts::TAU / longitudes as f64;
                    p.push([
                        (-s[0] / 2. * phi.cos() * theta.sin()) as f32 as f64,
                        (-s[0] / 2. * phi.sin() * theta.sin()) as f32 as f64,
                        (s[0] / 2. * theta.cos()) as f32 as f64,
                    ]);
                }
            }
        }
        _ => {
            for i in 0..n.segments {
                let a = i as f64 * std::f64::consts::TAU / n.segments as f64;
                p.push([s[0] / 2. * a.cos(), s[0] / 2. * a.sin(), -s[2] / 2.]);
                if !matches!(n.shape, Shape::Cone { .. }) {
                    p.push([s[0] / 2. * a.cos(), s[0] / 2. * a.sin(), s[2] / 2.]);
                }
            }
            if matches!(n.shape, Shape::Cone { .. }) {
                p.push([0., 0., s[2] / 2.]);
            }
        }
    }
    p
}
fn mv(v: Vec3) -> MVec {
    MVec::new(v[0], v[1], v[2])
}
pub fn primitive(n: &Node) -> Manifold {
    match &n.shape {
        Shape::Tube { .. } | Shape::Torus { .. } | Shape::Polygon { .. } | Shape::Text { .. } => {
            crate::catalog::primitive(n)
        }
        Shape::Box { size } => Manifold::cube(mv(*size), true),
        Shape::Cylinder { size } => Manifold::cylinder_centered(
            size[2],
            size[0] / 2.,
            size[0] / 2.,
            n.segments as i32,
            true,
        ),
        Shape::Mesh { triangles, .. } => crate::stl::manifold(&points(n), triangles),
        _ => Manifold::hull(&points(n).into_iter().map(mv).collect::<Vec<_>>()),
    }
}
pub fn mesh(m: &Manifold) -> Mesh {
    let raw = m.get_mesh_gl64(-1);
    Mesh {
        vertices: raw
            .vert_properties
            .chunks(raw.num_prop as usize)
            .map(|p| [p[0], p[1], p[2]])
            .collect(),
        triangles: raw
            .tri_verts
            .chunks(3)
            .map(|t| [t[0] as usize, t[1] as usize, t[2] as usize])
            .collect(),
    }
}
fn scope(nodes: &[Node]) -> Result<Manifold> {
    let mut solids = vec![];
    let mut holes = vec![];
    for n in nodes {
        let base = if let Shape::Group { scale, children } = &n.shape {
            scope(children)?.scale(MVec::new(*scale, *scale, *scale))
        } else {
            primitive(n)
        };
        let located = base
            .rotate(n.rotation[0], n.rotation[1], n.rotation[2])
            .translate(mv(n.position));
        ensure!(
            located.status() == Error::NoError,
            "Geometry construction failed: {:?}",
            located.status()
        );
        if n.hole {
            holes.push(located)
        } else {
            solids.push(located)
        }
    }
    let mut iter = solids.into_iter();
    let Some(mut result) = iter.next() else {
        return Ok(Manifold::empty());
    };
    for s in iter {
        result = result.union(&s);
    }
    for h in holes {
        result = result.difference(&h);
    }
    ensure!(
        result.status() == Error::NoError,
        "Boolean failed: {:?}",
        result.status()
    );
    Ok(result)
}
pub fn evaluate(p: &Project) -> Result<Mesh> {
    p.validate()?;
    let solid = scope(&p.parts)?;
    ensure!(
        !solid.is_empty(),
        "No printable solid: add a solid or reduce holes"
    );
    let m = mesh(&solid);
    ensure!(m.volume() > 0., "Invalid solid volume");
    Ok(m)
}
impl Mesh {
    pub fn transformed(&self, m: DMat4) -> Self {
        Self {
            vertices: self
                .vertices
                .iter()
                .map(|p| m.transform_point3(DVec3::from(*p)).to_array())
                .collect(),
            triangles: self.triangles.clone(),
        }
    }
    pub fn volume(&self) -> f64 {
        self.triangles
            .iter()
            .map(|t| {
                let a = DVec3::from(self.vertices[t[0]]);
                let b = DVec3::from(self.vertices[t[1]]);
                let c = DVec3::from(self.vertices[t[2]]);
                a.dot(b.cross(c)) / 6.
            })
            .sum()
    }
    pub fn stl(&self) -> Result<Vec<u8>> {
        ensure!(
            self.triangles.len() <= u32::MAX as usize,
            "Too many triangles"
        );
        let mut b = vec![0; 80];
        let label = b"TinkerCraft Rust | millimeter coordinates";
        b[..label.len()].copy_from_slice(label);
        b.extend((self.triangles.len() as u32).to_le_bytes());
        for t in &self.triangles {
            let v = t.map(|i| DVec3::from(self.vertices[i]));
            let normal = (v[1] - v[0]).cross(v[2] - v[0]);
            ensure!(normal.length() > 1e-16, "Degenerate export triangle");
            for p in [normal.normalize(), v[0], v[1], v[2]] {
                for n in p.to_array() {
                    ensure!(n.is_finite(), "Nonfinite export coordinate");
                    b.extend((n as f32).to_le_bytes());
                }
            }
            b.extend(0u16.to_le_bytes());
        }
        Ok(b)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub min: DVec3,
    pub max: DVec3,
}
impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: DVec3::splat(f64::INFINITY),
            max: DVec3::splat(f64::NEG_INFINITY),
        }
    }
}
impl Bounds {
    pub fn center(&self) -> DVec3 {
        (self.min + self.max) / 2.
    }
    pub fn add(&mut self, p: DVec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }
    pub fn merge(&mut self, b: Self) {
        self.add(b.min);
        self.add(b.max);
    }
}
pub fn bounds(n: &Node) -> Bounds {
    fn walk(n: &Node, parent: DMat4, b: &mut Bounds) {
        let m = parent * n.matrix();
        if let Shape::Group { children, .. } = &n.shape {
            for c in children.iter().filter(|c| !c.hole) {
                walk(c, m, b)
            }
        } else {
            for p in points(n) {
                b.add(m.transform_point3(DVec3::from(p)));
            }
        }
    }
    let mut b = Bounds::default();
    walk(n, DMat4::IDENTITY, &mut b);
    b
}
#[derive(Clone)]
pub struct SourceMesh {
    pub id: String,
    pub hole: bool,
    pub color: [u8; 3],
    pub mesh: Mesh,
}
pub fn sources(p: &Project) -> Vec<SourceMesh> {
    fn walk(n: &Node, parent: DMat4, root: &str, hole: bool, out: &mut Vec<SourceMesh>) {
        if n.hidden {
            return;
        }
        let m = parent * n.matrix();
        let hole = hole || n.hole;
        if let Shape::Group { children, .. } = &n.shape {
            for c in children {
                walk(c, m, root, hole, out)
            }
        } else {
            out.push(SourceMesh {
                id: root.to_string(),
                hole,
                color: n.color,
                mesh: mesh(&primitive(n)).transformed(m),
            });
        }
    }
    let mut out = vec![];
    for n in &p.parts {
        walk(n, DMat4::IDENTITY, &n.id, false, &mut out);
    }
    out
}

/// Editable top-level objects. Each group displays its scoped Boolean geometry
/// while retaining its object ID for picking, selection and transform previews.
/// Hidden children still participate in the group result, just as in export.
pub fn editable_sources(p: &Project) -> Result<Vec<SourceMesh>> {
    p.validate()?;
    let mut out = Vec::new();
    for n in p.parts.iter().filter(|n| !n.hidden) {
        for (solid, color) in colored_node(n)? {
            if !solid.is_empty() {
                out.push(SourceMesh {
                    id: n.id.clone(),
                    hole: n.hole,
                    color,
                    mesh: mesh(&solid),
                });
            }
        }
    }
    Ok(out)
}

#[derive(Clone)]
pub struct ColoredMesh {
    pub color: [u8; 3],
    pub mesh: Mesh,
}

fn checked(m: Manifold) -> Result<Manifold> {
    ensure!(
        m.status() == Error::NoError,
        "Color Boolean failed: {:?}",
        m.status()
    );
    Ok(m)
}
fn colored_node(n: &Node) -> Result<Vec<(Manifold, [u8; 3])>> {
    let pieces = match &n.shape {
        Shape::Group { children, .. } => colored_scope(children)?,
        _ => vec![(primitive(n), n.color)],
    };
    pieces
        .into_iter()
        .map(|(m, c)| {
            let m = m
                .scale(MVec::new(n.scale(), n.scale(), n.scale()))
                .rotate(n.rotation[0], n.rotation[1], n.rotation[2])
                .translate(mv(n.position));
            Ok((checked(m)?, c))
        })
        .collect()
}
// Later solids own overlapping volume. Cuts apply to every color in their scope.
// Parts therefore form a disjoint partition of the ordinary Boolean result.
fn colored_scope(nodes: &[Node]) -> Result<Vec<(Manifold, [u8; 3])>> {
    let mut parts: Vec<(Manifold, [u8; 3])> = Vec::new();
    let mut holes = Vec::new();
    for n in nodes {
        let incoming = colored_node(n)?;
        if n.hole {
            holes.extend(incoming.into_iter().map(|(m, _)| m));
            continue;
        }
        for (solid, color) in incoming {
            for (previous, _) in &mut parts {
                *previous = checked(previous.difference(&solid))?;
            }
            parts.retain(|(m, _)| !m.is_empty());
            if !solid.is_empty() {
                parts.push((solid, color));
            }
        }
    }
    for hole in holes {
        for (part, _) in &mut parts {
            *part = checked(part.difference(&hole))?;
        }
        parts.retain(|(m, _)| !m.is_empty());
    }
    Ok(parts)
}
pub fn colored_parts(p: &Project) -> Result<Vec<ColoredMesh>> {
    p.validate()?;
    let pieces = colored_scope(&p.parts)?;
    ensure!(
        !pieces.is_empty(),
        "No printable solid: add a solid or reduce holes"
    );
    Ok(pieces
        .into_iter()
        .map(|(solid, color)| ColoredMesh {
            color,
            mesh: mesh(&solid),
        })
        .collect())
}
