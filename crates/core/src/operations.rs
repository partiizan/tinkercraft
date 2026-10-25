use crate::{
    document::*,
    geometry::{Bounds, bounds},
};
use anyhow::{Result, ensure};
use glam::{DMat3, DMat4, DQuat, DVec3};
use std::collections::HashSet;
pub type Selection = HashSet<String>;
fn selected_bounds(p: &Project, ids: &Selection) -> Result<Bounds> {
    let mut b = Bounds::default();
    let mut count = 0;
    for n in p.parts.iter().filter(|n| ids.contains(&n.id)) {
        b.merge(bounds(n));
        count += 1;
    }
    ensure!(count > 0, "Select an object");
    Ok(b)
}
pub fn group(p: &mut Project, ids: &Selection) -> Result<String> {
    let selected: Vec<_> = p
        .parts
        .iter()
        .filter(|n| ids.contains(&n.id))
        .cloned()
        .collect();
    ensure!(
        selected.len() >= 2 && selected.iter().any(|n| !n.hole),
        "Select 2+ objects including a solid"
    );
    let pivot = selected_bounds(p, ids)?.center();
    let index = p.parts.iter().position(|n| ids.contains(&n.id)).unwrap();
    let children = selected
        .into_iter()
        .map(|mut n| {
            n.position = (DVec3::from(n.position) - pivot).to_array();
            n
        })
        .collect();
    p.parts.retain(|n| !ids.contains(&n.id));
    let id = uuid::Uuid::new_v4().to_string();
    p.parts.insert(
        index,
        Node {
            id: id.clone(),
            name: "Group".into(),
            hole: false,
            color: crate::document::default_color(),
            segments: crate::document::default_segments(),
            hidden: false,
            locked: false,
            position: pivot.to_array(),
            rotation: [0.; 3],
            shape: Shape::Group {
                scale: 1.,
                children,
            },
        },
    );
    Ok(id)
}
pub fn ungroup(p: &mut Project, id: &str) -> Result<Selection> {
    let index = p
        .parts
        .iter()
        .position(|n| n.id == id)
        .ok_or_else(|| anyhow::anyhow!("Select a group"))?;
    let n = p.parts[index].clone();
    ensure!(!n.hole, "Make the group Solid before ungrouping");
    let Shape::Group {
        scale,
        mut children,
    } = n.shape
    else {
        anyhow::bail!("Select a group")
    };
    let q = quaternion(n.rotation);
    let mut ids = Selection::new();
    for child in &mut children {
        child.position =
            (q * (DVec3::from(child.position) * scale) + DVec3::from(n.position)).to_array();
        child.rotation = rotation(q * quaternion(child.rotation));
        match &mut child.shape {
            Shape::Group { scale: s, .. } => *s *= scale,
            Shape::Box { size }
            | Shape::Cylinder { size }
            | Shape::Cone { size }
            | Shape::Sphere { size }
            | Shape::Wedge { size }
            | Shape::Roof { size }
            | Shape::Mesh { size, .. } => *size = size.map(|s| s * scale),
        }
        ids.insert(child.id.clone());
    }
    p.parts.splice(index..=index, children);
    Ok(ids)
}
pub fn duplicate(p: &mut Project, ids: &Selection) -> Selection {
    fn fresh(n: &mut Node) {
        n.id = uuid::Uuid::new_v4().to_string();
        if let Shape::Group { children, .. } = &mut n.shape {
            for c in children {
                fresh(c)
            }
        }
    }
    let mut copies: Vec<_> = p
        .parts
        .iter()
        .filter(|n| ids.contains(&n.id))
        .cloned()
        .collect();
    let mut selected = Selection::new();
    for n in &mut copies {
        fresh(n);
        n.position[0] += 10.;
        n.name = format!("{} copy", n.name.chars().take(95).collect::<String>());
        selected.insert(n.id.clone());
    }
    p.parts.extend(copies);
    selected
}
pub fn align(p: &mut Project, ids: &Selection, axis: usize, mode: i32) -> Result<()> {
    ensure!(axis < 3 && (-1..=1).contains(&mode), "Invalid alignment");
    ensure!(
        p.parts.iter().filter(|n| ids.contains(&n.id)).count() >= 2,
        "Select at least two objects"
    );
    let b = selected_bounds(p, ids)?;
    let target = if mode < 0 {
        b.min[axis]
    } else if mode > 0 {
        b.max[axis]
    } else {
        b.center()[axis]
    };
    for n in p.parts.iter_mut().filter(|n| ids.contains(&n.id)) {
        let b = bounds(n);
        let current = if mode < 0 {
            b.min[axis]
        } else if mode > 0 {
            b.max[axis]
        } else {
            b.center()[axis]
        };
        n.position[axis] += target - current;
    }
    Ok(())
}
pub fn mirror(p: &mut Project, ids: &Selection, axis: usize) -> Result<()> {
    ensure!(axis < 3, "Invalid axis");
    let plane = selected_bounds(p, ids)?.center()[axis];
    fn reflect(n: &mut Node, axis: usize, plane: f64) {
        let mut sign = DVec3::ONE;
        sign[axis] = -1.;
        let s = DMat3::from_diagonal(sign);
        let mut r = s * DMat3::from_quat(quaternion(n.rotation)) * s;
        if matches!(n.shape, Shape::Cone { .. } | Shape::Roof { .. }) && axis == 2 {
            r *= DMat3::from_rotation_x(std::f64::consts::PI);
        }
        if matches!(n.shape, Shape::Wedge { .. }) && axis != 1 {
            // Wedge is symmetric only about local Y. Compose two reflections
            // into a proper rotation to preserve its sloping face.
            r *= s * DMat3::from_diagonal(DVec3::new(1., -1., 1.));
        }
        if let Shape::Mesh {
            vertices,
            triangles,
            ..
        } = &mut n.shape
        {
            for v in vertices {
                v[axis] = -v[axis];
            }
            for t in triangles {
                t.swap(1, 2);
            }
        }
        n.rotation = rotation(DQuat::from_mat3(&r));
        n.position[axis] = 2. * plane - n.position[axis];
        if let Shape::Group { children, .. } = &mut n.shape {
            for c in children {
                reflect(c, axis, 0.)
            }
        }
    }
    for n in p.parts.iter_mut().filter(|n| ids.contains(&n.id)) {
        reflect(n, axis, plane);
    }
    Ok(())
}
pub fn drop_to_plane(p: &mut Project, ids: &Selection) -> Result<()> {
    for n in p.parts.iter_mut().filter(|n| ids.contains(&n.id)) {
        n.position[2] -= bounds(n).min.z;
    }
    Ok(())
}
pub fn world_rotation(n: &mut Node, axis: usize, degrees: f64) {
    let q = DQuat::from_axis_angle([DVec3::X, DVec3::Y, DVec3::Z][axis], degrees.to_radians());
    n.rotation = rotation(q * quaternion(n.rotation));
}
pub fn world_matrix(n: &Node) -> DMat4 {
    n.matrix()
}

/// Dimensions in the object's local axes, including uniform group scale.
pub fn local_dimensions(n: &Node) -> Vec3 {
    if let Some(size) = n.size() {
        return size;
    }
    let mut local = n.clone();
    local.position = [0.; 3];
    local.rotation = [0.; 3];
    let b = bounds(&local);
    (b.max - b.min).to_array()
}
/// Center/pivot anchored resize. Circular primitives stay circular; groups scale uniformly.
pub fn resize_node(n: &Node, axis: usize, dimension: f64) -> Result<Node> {
    ensure!(
        axis < 3 && dimension.is_finite() && dimension >= 0.01,
        "Invalid resize dimension"
    );
    let mut result = n.clone();
    if let Shape::Group { scale, .. } = &mut result.shape {
        let old = local_dimensions(n)[axis];
        ensure!(old > 0., "Cannot resize empty bounds");
        *scale *= dimension / old;
    } else {
        result.set_dimension(axis, dimension);
    }
    Project {
        parts: vec![result.clone()],
        ..Project::default()
    }
    .validate()?;
    Ok(result)
}
/// Affine source-preview update, identical to the committed size/transform semantics.
pub fn resize_preview_matrix(before: &Node, after: &Node) -> DMat4 {
    let scale = match (before.size(), after.size()) {
        (Some(a), Some(b)) => DVec3::from(b) / DVec3::from(a),
        _ => DVec3::ONE,
    };
    after.matrix() * DMat4::from_scale(scale) * before.matrix().inverse()
}

/// Lock protection is enforced at history commit, including keyboard and compound actions.
/// A locked descendant protects its entire top-level group against modeling edits.
pub fn protect_locked(before: &Project, after: &Project) -> Result<()> {
    fn without_flags(n: &Node) -> Node {
        let mut n = n.clone();
        n.hidden = false;
        n.locked = false;
        if let Shape::Group { children, .. } = &mut n.shape {
            *children = children.iter().map(without_flags).collect();
        }
        n
    }
    for old in before.parts.iter().filter(|n| n.has_locked()) {
        let new = after.parts.iter().find(|n| n.id == old.id);
        ensure!(
            new.is_some_and(|n| without_flags(n) == without_flags(old)),
            "Unlock the object/group before editing or deleting it"
        );
    }
    Ok(())
}

/// World-axis nudge; the history boundary protects locked nodes.
pub fn translate(p: &mut Project, ids: &Selection, delta: Vec3) -> Result<()> {
    ensure!(delta.iter().all(|v| v.is_finite()), "Invalid movement");
    for n in p.parts.iter_mut().filter(|n| ids.contains(&n.id)) {
        n.position = (DVec3::from(n.position) + DVec3::from(delta)).to_array();
    }
    p.validate()
}
