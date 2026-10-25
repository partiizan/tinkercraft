use anyhow::{Result, ensure};
use glam::{DMat4, DQuat, DVec3, EulerRot};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
pub type Vec3 = [f64; 3];
pub const MAX_BYTES: usize = 32_000_000;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Shape {
    Box {
        size: Vec3,
    },
    Cylinder {
        size: Vec3,
    },
    Sphere {
        size: Vec3,
    },
    Cone {
        size: Vec3,
    },
    Wedge {
        size: Vec3,
    },
    Roof {
        size: Vec3,
    },
    Mesh {
        size: Vec3,
        vertices: Vec<Vec3>,
        triangles: Vec<[usize; 3]>,
    },
    Group {
        scale: f64,
        children: Vec<Node>,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub hole: bool,
    #[serde(default = "default_color")]
    pub color: [u8; 3],
    #[serde(default = "default_segments")]
    pub segments: u32,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub locked: bool,
    pub position: Vec3,
    #[serde(default)]
    pub rotation: Vec3,
    #[serde(flatten)]
    pub shape: Shape,
}
pub fn default_segments() -> u32 {
    96
}
pub fn default_color() -> [u8; 3] {
    [141, 190, 206]
}
impl Node {
    pub fn set_color_recursive(&mut self, color: [u8; 3]) {
        self.color = color;
        if let Shape::Group { children, .. } = &mut self.shape {
            for child in children {
                child.set_color_recursive(color);
            }
        }
    }
    pub fn has_locked(&self) -> bool {
        self.locked
            || matches!(&self.shape, Shape::Group { children, .. } if children.iter().any(Node::has_locked))
    }
    pub fn set_locked_recursive(&mut self, value: bool) {
        self.locked = value;
        if let Shape::Group { children, .. } = &mut self.shape {
            for c in children {
                c.set_locked_recursive(value);
            }
        }
    }

    pub fn primitive(kind: &str) -> Self {
        let size = [20.; 3];
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: kind.to_owned(),
            hole: false,
            color: default_color(),
            segments: default_segments(),
            hidden: false,
            locked: false,
            position: [40., 15., 10.],
            rotation: [0.; 3],
            shape: match kind {
                "Cylinder" => Shape::Cylinder { size },
                "Sphere" => Shape::Sphere { size },
                "Cone" => Shape::Cone { size },
                "Wedge" => Shape::Wedge { size },
                "Roof" => Shape::Roof { size },
                _ => Shape::Box { size },
            },
        }
    }
    pub fn size(&self) -> Option<Vec3> {
        match &self.shape {
            Shape::Box { size }
            | Shape::Cylinder { size }
            | Shape::Sphere { size }
            | Shape::Cone { size }
            | Shape::Wedge { size }
            | Shape::Roof { size }
            | Shape::Mesh { size, .. } => Some(*size),
            _ => None,
        }
    }
    pub fn set_dimension(&mut self, axis: usize, value: f64) {
        match &mut self.shape {
            Shape::Box { size }
            | Shape::Wedge { size }
            | Shape::Roof { size }
            | Shape::Mesh { size, .. } => size[axis] = value,
            Shape::Cylinder { size } | Shape::Cone { size } => {
                size[axis] = value;
                if axis < 2 {
                    size[0] = value;
                    size[1] = value;
                }
            }
            Shape::Sphere { size } => *size = [value; 3],
            _ => {}
        }
    }
    pub fn scale(&self) -> f64 {
        if let Shape::Group { scale, .. } = &self.shape {
            *scale
        } else {
            1.
        }
    }
    pub fn matrix(&self) -> DMat4 {
        DMat4::from_scale_rotation_translation(
            DVec3::splat(self.scale()),
            quaternion(self.rotation),
            DVec3::from(self.position),
        )
    }
    pub fn kind(&self) -> &str {
        match &self.shape {
            Shape::Box { .. } => "Box",
            Shape::Cylinder { .. } => "Cylinder",
            Shape::Sphere { .. } => "Sphere",
            Shape::Cone { .. } => "Cone",
            Shape::Group { .. } => "Group",
            Shape::Mesh { .. } => "Mesh",
            Shape::Wedge { .. } => "Wedge",
            Shape::Roof { .. } => "Roof",
        }
    }
}
pub fn quaternion(r: Vec3) -> DQuat {
    DQuat::from_euler(
        EulerRot::ZYX,
        r[2].to_radians(),
        r[1].to_radians(),
        r[0].to_radians(),
    )
}
pub fn rotation(q: DQuat) -> Vec3 {
    let (z, y, x) = q.normalize().to_euler(EulerRot::ZYX);
    [x, y, z].map(|v| {
        let r = (v.to_degrees() * 1e9).round() / 1e9;
        if r == 0. { 0. } else { r }
    })
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub format: String,
    pub version: u32,
    pub units: String,
    pub name: String,
    pub parts: Vec<Node>,
}
impl Default for Project {
    fn default() -> Self {
        Self {
            format: "shapeworkshop".into(),
            version: 9,
            units: "mm".into(),
            name: "Untitled".into(),
            parts: vec![],
        }
    }
}
fn text(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.chars().count() <= max && !s.chars().any(|c| c < ' ')
}
impl Project {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.format == "shapeworkshop" && self.version == 9 && self.units == "mm",
            "Unsupported project format/version/units"
        );
        ensure!(text(&self.name, 100), "Invalid project name");
        let mut ids = HashSet::new();
        let mut count = 0;
        fn walk(
            n: &Node,
            depth: usize,
            parent: DMat4,
            scale: f64,
            ids: &mut HashSet<String>,
            count: &mut usize,
        ) -> Result<()> {
            *count += 1;
            ensure!(
                *count <= 100 && depth <= 8,
                "Limit: 100 nodes and 8 group levels"
            );
            ensure!(
                text(&n.id, 128) && ids.insert(n.id.clone()),
                "Invalid or duplicate object ID"
            );
            ensure!(text(&n.name, 100), "Invalid object name");
            ensure!(
                (24..=192).contains(&n.segments) && n.segments.is_multiple_of(4),
                "Curve segments must be 24–192 in multiples of four"
            );
            ensure!(
                n.position
                    .iter()
                    .all(|v| v.is_finite() && v.abs() <= 10000.),
                "Position outside ±10000 mm"
            );
            ensure!(
                n.rotation.iter().all(|v| v.is_finite() && v.abs() <= 3600.),
                "Rotation outside ±3600 degrees"
            );
            let world = parent * n.matrix();
            ensure!(
                world
                    .transform_point3(DVec3::ZERO)
                    .to_array()
                    .iter()
                    .all(|v| v.is_finite() && v.abs() <= 10000.00000001),
                "World center outside ±10000 mm"
            );
            if let Shape::Group { scale: s, children } = &n.shape {
                ensure!(
                    s.is_finite() && *s >= 0.00001 && *s <= 100000.,
                    "Invalid group scale"
                );
                ensure!(
                    children.iter().any(|c| !c.hole),
                    "A group needs a solid child"
                );
                for c in children {
                    walk(c, depth + 1, world, scale * s, ids, count)?;
                }
            } else {
                let size = n.size().unwrap();
                ensure!(
                    size.iter().all(|v| v.is_finite()
                        && *v >= 0.01
                        && *v <= 1000.
                        && v * scale >= 0.01 - 1e-10
                        && v * scale <= 1000. + 1e-8),
                    "Dimensions must be 0.01–1000 mm locally and after scaling"
                );
                if matches!(
                    n.shape,
                    Shape::Cylinder { .. } | Shape::Cone { .. } | Shape::Sphere { .. }
                ) {
                    ensure!(size[0] == size[1], "Circular diameters must match");
                }
                if matches!(n.shape, Shape::Sphere { .. }) {
                    ensure!(size[0] == size[2], "Sphere diameters must match");
                }
            }
            if let Shape::Mesh {
                vertices,
                triangles,
                ..
            } = &n.shape
            {
                crate::stl::validate_mesh(vertices, triangles)?;
                ensure!(
                    vertices.iter().flatten().all(|v| v.abs() <= 0.50000001),
                    "Mesh coordinates must be normalized"
                );
            }
            Ok(())
        }
        for n in &self.parts {
            walk(n, 0, DMat4::IDENTITY, 1., &mut ids, &mut count)?;
        }
        fn triangles(n: &Node) -> usize {
            match &n.shape {
                Shape::Mesh { triangles, .. } => triangles.len(),
                Shape::Group { children, .. } => children.iter().map(triangles).sum(),
                _ => 0,
            }
        }
        ensure!(
            self.parts.iter().map(triangles).sum::<usize>() <= crate::stl::MAX_TRIANGLES,
            "Project imported-mesh limit: 20,000 triangles"
        );
        Ok(())
    }
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= MAX_BYTES, "Project exceeds 32 MB");
        let mut v: serde_json::Value = serde_json::from_slice(bytes)?;
        let version = v["version"].as_u64().unwrap_or(0);
        ensure!((1..=9).contains(&version), "Unsupported project version");
        fn migrate(v: &mut serde_json::Value, version: u64, depth: usize) -> Result<()> {
            ensure!(depth <= 8, "Group nesting limit exceeded");
            let kind = v["kind"].as_str().unwrap_or("");
            ensure!(
                version >= 7
                    || (!v["hidden"].as_bool().unwrap_or(false)
                        && !v["locked"].as_bool().unwrap_or(false)),
                "Hide/lock flags require schema 7"
            );
            ensure!(
                version >= 6 || !matches!(kind, "wedge" | "roof"),
                "Wedge/roof require schema 6"
            );
            ensure!(version >= 5 || kind != "mesh", "Mesh requires schema 5");
            ensure!(
                version >= 4 || !matches!(kind, "sphere" | "cone"),
                "Shape not valid in legacy schema"
            );
            ensure!(
                version >= 3 || kind != "group",
                "Group not valid in legacy schema"
            );
            if version == 1 && v.get("rotation").is_none() {
                v["rotation"] = serde_json::json!([0, 0, 0]);
            }
            ensure!(v.get("rotation").is_some(), "Missing rotation");
            if let Some(children) = v.get_mut("children").and_then(|x| x.as_array_mut()) {
                for child in children {
                    migrate(child, version, depth + 1)?;
                }
            }
            Ok(())
        }
        if let Some(parts) = v.get_mut("parts").and_then(|x| x.as_array_mut()) {
            for n in parts {
                migrate(n, version, 0)?;
            }
        }
        v["version"] = 9.into();
        let p: Self = serde_json::from_value(v)?;
        p.validate()?;
        crate::stl::validate_project_meshes(&p)?;
        Ok(p)
    }
    pub fn bytes(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let b = serde_json::to_vec_pretty(self)?;
        ensure!(b.len() <= MAX_BYTES, "Project exceeds 32 MB");
        Ok(b)
    }
    pub fn demo() -> Self {
        Self::parse(include_bytes!("../../../examples/plate.tinkercraft")).expect("bundled demo")
    }
}
