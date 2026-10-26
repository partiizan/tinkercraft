//! Workplanes, ray picking, point snapping and view-space selection.
use crate::{
    Project,
    document::{Vec3, quaternion, rotation},
    geometry::{self, SourceMesh},
    operations::Selection,
    viewport::Camera,
};
use anyhow::{Result, ensure};
use glam::{DMat3, DQuat, DVec2, DVec3};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Workplane {
    pub origin: Vec3,
    pub rotation: Vec3,
}
impl Workplane {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.origin
                .iter()
                .all(|v| v.is_finite() && v.abs() <= 10000.)
                && self
                    .rotation
                    .iter()
                    .all(|v| v.is_finite() && v.abs() <= 3600.),
            "Invalid workplane"
        );
        Ok(())
    }
    pub fn world(&self, p: DVec3) -> DVec3 {
        DVec3::from(self.origin) + quaternion(self.rotation) * p
    }
    pub fn local(&self, p: DVec3) -> DVec3 {
        quaternion(self.rotation).inverse() * (p - DVec3::from(self.origin))
    }
    pub fn normal(&self) -> DVec3 {
        quaternion(self.rotation) * DVec3::Z
    }
    pub fn on_face(origin: DVec3, normal: DVec3) -> Result<Self> {
        ensure!(
            origin.is_finite() && normal.is_finite() && normal.length() > 1e-9,
            "Invalid face"
        );
        let z = normal.normalize();
        let seed = if z.dot(DVec3::X).abs() < 0.9 {
            DVec3::X
        } else {
            DVec3::Y
        };
        let x = (seed - z * seed.dot(z)).normalize();
        let y = z.cross(x);
        let result = Self {
            origin: origin.to_array(),
            rotation: rotation(DQuat::from_mat3(&DMat3::from_cols(x, y, z))),
        };
        result.validate()?;
        Ok(result)
    }
    pub fn intersect(&self, origin: DVec3, direction: DVec3, snap: f64) -> Option<DVec3> {
        let o = self.local(origin);
        let d = quaternion(self.rotation).inverse() * direction;
        let local = crate::modeling::workplane_point(o, d, snap)?;
        let p = self.world(local);
        (p.is_finite() && p.abs().max_element() <= 10000.).then_some(p)
    }
}
#[derive(Clone, Debug)]
pub struct Hit {
    pub id: String,
    pub point: DVec3,
    pub normal: DVec3,
    pub vertices: [DVec3; 3],
    pub distance: f64,
}
pub fn pick(sources: &[SourceMesh], origin: DVec3, direction: DVec3) -> Option<Hit> {
    if !origin.is_finite() || !direction.is_finite() || direction.length() < 1e-12 {
        return None;
    }
    let d = direction.normalize();
    let mut hit: Option<Hit> = None;
    for s in sources {
        for t in &s.mesh.triangles {
            let [a, b, c] = t.map(|i| DVec3::from(s.mesh.vertices[i]));
            let e1 = b - a;
            let e2 = c - a;
            let h = d.cross(e2);
            let det = e1.dot(h);
            if det.abs() < 1e-10 {
                continue;
            }
            let inv = 1. / det;
            let q = origin - a;
            let u = q.dot(h) * inv;
            if !(0. ..=1.).contains(&u) {
                continue;
            }
            let r = q.cross(e1);
            let v = d.dot(r) * inv;
            if v < 0. || u + v > 1. {
                continue;
            }
            let distance = e2.dot(r) * inv;
            if distance > 1e-7 && hit.as_ref().is_none_or(|h| distance < h.distance) {
                hit = Some(Hit {
                    id: s.id.clone(),
                    point: origin + d * distance,
                    normal: e1.cross(e2).normalize(),
                    vertices: [a, b, c],
                    distance,
                });
            }
        }
    }
    hit
}
pub fn snap_point(hit: &Hit, vertex: bool) -> DVec3 {
    if vertex {
        *hit.vertices
            .iter()
            .min_by(|a, b| {
                a.distance_squared(hit.point)
                    .total_cmp(&b.distance_squared(hit.point))
            })
            .unwrap()
    } else {
        hit.point
    }
}
pub fn drop_to_workplane(p: &mut Project, ids: &Selection) -> Result<()> {
    let mut next = p.clone();
    let plane = &p.workplane;
    let mut count = 0;
    for n in next.parts.iter_mut().filter(|n| ids.contains(&n.id)) {
        ensure!(
            !n.hidden && !n.has_locked(),
            "Show and unlock objects first"
        );
        let mut solid = n.clone();
        solid.hole = false;
        let sources = geometry::sources(&Project {
            parts: vec![solid],
            ..Project::default()
        });
        let distance = sources
            .iter()
            .filter(|s| !s.hole)
            .flat_map(|s| s.mesh.vertices.iter())
            .map(|v| plane.local(DVec3::from(*v)).z)
            .fold(f64::INFINITY, f64::min);
        ensure!(distance.is_finite(), "Object has no source bounds");
        n.position = (DVec3::from(n.position) - plane.normal() * distance).to_array();
        count += 1;
    }
    ensure!(count == ids.len() && count > 0, "Select existing objects");
    next.validate()?;
    crate::operations::protect_locked(p, &next)?;
    *p = next;
    Ok(())
}
pub fn box_select(
    p: &Project,
    camera: &Camera,
    width: usize,
    height: usize,
    a: DVec2,
    b: DVec2,
) -> Selection {
    let min = a.min(b);
    let max = a.max(b);
    p.parts
        .iter()
        .filter(|n| !n.hidden && !n.has_locked())
        .filter(|n| {
            let bounds = geometry::bounds(n);
            camera
                .project(bounds.center(), width, height)
                .is_some_and(|q| q[0] >= min.x && q[0] <= max.x && q[1] >= min.y && q[1] <= max.y)
        })
        .map(|n| n.id.clone())
        .collect()
}
/// Replace one group's contents in a parent scene, retaining the group's identity/transform.
pub fn replace_children(parent: &Project, id: &str, children: Vec<crate::Node>) -> Result<Project> {
    let mut p = parent.clone();
    let n = p
        .parts
        .iter_mut()
        .find(|n| n.id == id)
        .ok_or_else(|| anyhow::anyhow!("Group no longer exists"))?;
    ensure!(!n.hidden && !n.has_locked(), "Show and unlock group first");
    let crate::Shape::Group { children: c, .. } = &mut n.shape else {
        anyhow::bail!("Select a group")
    };
    *c = children;
    p.validate()?;
    crate::operations::protect_locked(parent, &p)?;
    Ok(p)
}
