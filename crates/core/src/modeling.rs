//! Modeling tools. Measurements use world-axis source bounds, before Boolean cuts.
use crate::{
    Node, Project, Shape,
    geometry::{Bounds, bounds},
    operations::{self, Selection},
};
use anyhow::{Result, ensure};
use glam::{DQuat, DVec3};

pub fn selection_bounds(p: &Project, ids: &Selection) -> Option<Bounds> {
    let mut result = Bounds::default();
    for n in p.parts.iter().filter(|n| ids.contains(&n.id)) {
        result.merge(bounds(n));
    }
    (result.min.is_finite() && result.max.is_finite()).then_some(result)
}
pub fn bounds_gap(a: &Bounds, b: &Bounds) -> DVec3 {
    (a.min - b.max).max(b.min - a.max).max(DVec3::ZERO)
}
pub fn workplane_point(origin: DVec3, direction: DVec3, snap: f64) -> Option<DVec3> {
    if !origin.is_finite() || !direction.is_finite() || direction.z.abs() < 1e-9 {
        return None;
    }
    let t = -origin.z / direction.z;
    if t < 0. || !t.is_finite() {
        return None;
    }
    let mut p = origin + direction * t;
    if snap.is_finite() && snap > 0. {
        p = (p / snap).round() * snap;
    }
    p.z = 0.;
    (p.is_finite() && p.abs().max_element() <= 10000.).then_some(p)
}
fn selected(p: &Project, ids: &Selection) -> Result<Vec<usize>> {
    let indices: Vec<_> = p
        .parts
        .iter()
        .enumerate()
        .filter(|(_, n)| ids.contains(&n.id))
        .map(|(i, _)| i)
        .collect();
    ensure!(
        !indices.is_empty() && indices.len() == ids.len(),
        "Select existing objects"
    );
    ensure!(
        indices
            .iter()
            .all(|&i| !p.parts[i].hidden && !p.parts[i].has_locked()),
        "Show and unlock the selection before editing"
    );
    Ok(indices)
}
/// Equal center spacing or equal source-bounding-box gaps. End objects remain fixed.
pub fn distribute(p: &mut Project, ids: &Selection, axis: usize, gaps: bool) -> Result<()> {
    ensure!(axis < 3, "Invalid axis");
    let indices = selected(p, ids)?;
    ensure!(indices.len() >= 3, "Select at least three objects");
    let mut items: Vec<_> = indices
        .into_iter()
        .map(|i| (i, bounds(&p.parts[i])))
        .collect();
    ensure!(
        items
            .iter()
            .all(|(_, b)| b.min.is_finite() && b.max.is_finite()),
        "Selection has no finite bounds"
    );
    items.sort_by(|a, b| {
        a.1.center()[axis]
            .total_cmp(&b.1.center()[axis])
            .then(a.0.cmp(&b.0))
    });
    let first = &items[0].1;
    let last = &items[items.len() - 1].1;
    let step = if gaps {
        (last.max[axis]
            - first.min[axis]
            - items
                .iter()
                .map(|(_, b)| b.max[axis] - b.min[axis])
                .sum::<f64>())
            / (items.len() - 1) as f64
    } else {
        (last.center()[axis] - first.center()[axis]) / (items.len() - 1) as f64
    };
    let mut next = p.clone();
    let mut cursor = first.min[axis];
    for (j, (i, b)) in items.iter().enumerate() {
        let target = if gaps {
            cursor + (b.max[axis] - b.min[axis]) / 2.
        } else {
            first.center()[axis] + step * j as f64
        };
        if j > 0 && j + 1 < items.len() {
            next.parts[*i].position[axis] += target - b.center()[axis];
        }
        cursor += b.max[axis] - b.min[axis] + step;
    }
    next.validate()?;
    operations::protect_locked(p, &next)?;
    *p = next;
    Ok(())
}
#[derive(Clone, Copy)]
pub enum Repeat {
    Linear([f64; 3]),
    Radial {
        pivot: [f64; 3],
        degrees: f64,
        rotate: bool,
    },
}
fn count(n: &Node) -> usize {
    1 + match &n.shape {
        Shape::Group { children, .. } => children.iter().map(count).sum(),
        _ => 0,
    }
}
fn fresh(n: &mut Node) {
    n.id = uuid::Uuid::new_v4().to_string();
    if let Shape::Group { children, .. } = &mut n.shape {
        for c in children {
            fresh(c)
        }
    }
}
pub fn repeat(
    p: &mut Project,
    ids: &Selection,
    copies: usize,
    pattern: Repeat,
) -> Result<Selection> {
    let indices = selected(p, ids)?;
    ensure!((1..=99).contains(&copies), "Use 1–99 additional copies");
    let added: usize = indices.iter().map(|&i| count(&p.parts[i])).sum();
    ensure!(
        p.parts.iter().map(count).sum::<usize>() + copies * added <= 100,
        "Copies exceed the 100-node project limit"
    );
    match pattern {
        Repeat::Linear(v) => ensure!(v.iter().all(|x| x.is_finite()), "Invalid spacing"),
        Repeat::Radial { pivot, degrees, .. } => ensure!(
            pivot.iter().all(|x| x.is_finite())
                && degrees.is_finite()
                && degrees.abs() > 0.
                && degrees.abs() <= 360.,
            "Use a nonzero sweep up to 360 degrees"
        ),
    }
    let mut next = p.clone();
    let mut created = Selection::new();
    for k in 1..=copies {
        for &i in &indices {
            let mut n = p.parts[i].clone();
            fresh(&mut n);
            match pattern {
                Repeat::Linear(v) => {
                    n.position = (DVec3::from(n.position) + DVec3::from(v) * k as f64).to_array()
                }
                Repeat::Radial {
                    pivot,
                    degrees,
                    rotate,
                } => {
                    let denominator = if (degrees.abs() - 360.).abs() < 1e-8 {
                        copies + 1
                    } else {
                        copies
                    };
                    let q = DQuat::from_rotation_z(
                        (degrees * k as f64 / denominator as f64).to_radians(),
                    );
                    let center = DVec3::from(pivot);
                    n.position = (center + q * (DVec3::from(n.position) - center)).to_array();
                    if rotate {
                        n.rotation =
                            crate::document::rotation(q * crate::document::quaternion(n.rotation));
                    }
                }
            }
            created.insert(n.id.clone());
            next.parts.push(n);
        }
    }
    next.validate()?;
    operations::protect_locked(p, &next)?;
    *p = next;
    Ok(created)
}
/// Replace a staged group atomically without flattening its children or moving its pivot.
pub fn edit_group(p: &mut Project, original: &Node, edited: Node) -> Result<()> {
    let i = p
        .parts
        .iter()
        .position(|n| n.id == original.id)
        .ok_or_else(|| anyhow::anyhow!("Group no longer exists"))?;
    ensure!(
        p.parts[i] == *original,
        "Group changed; reopen its contents"
    );
    ensure!(
        !original.has_locked() && !original.hidden,
        "Show and unlock the group first"
    );
    ensure!(
        edited.id == original.id
            && matches!(original.shape, Shape::Group { .. })
            && matches!(edited.shape, Shape::Group { .. }),
        "Expected the same group"
    );
    ensure!(
        edited.position == original.position
            && edited.rotation == original.rotation
            && edited.scale() == original.scale(),
        "Edit contents must preserve the group transform"
    );
    let mut next = p.clone();
    next.parts[i] = edited;
    next.validate()?;
    operations::protect_locked(p, &next)?;
    *p = next;
    Ok(())
}

/// Align selected objects to one stationary reference, using world source bounds.
/// The reference may be locked; only the moving objects must be editable.
pub fn align_reference(
    p: &mut Project,
    ids: &Selection,
    reference: &str,
    axis: usize,
    source_mode: i32,
    target_mode: i32,
    offset: f64,
) -> Result<()> {
    ensure!(
        axis < 3
            && (-1..=1).contains(&source_mode)
            && (-1..=1).contains(&target_mode)
            && offset.is_finite(),
        "Invalid reference alignment"
    );
    let anchor = p
        .parts
        .iter()
        .find(|n| n.id == reference && !n.hidden)
        .ok_or_else(|| anyhow::anyhow!("Choose a visible reference object"))?;
    let a = bounds(anchor);
    let coordinate = |b: Bounds, mode: i32| {
        if mode < 0 {
            b.min[axis]
        } else if mode > 0 {
            b.max[axis]
        } else {
            b.center()[axis]
        }
    };
    let target = coordinate(a, target_mode) + offset;
    let moving: Selection = ids
        .iter()
        .filter(|id| id.as_str() != reference)
        .cloned()
        .collect();
    let indices = selected(p, &moving)?;
    let mut next = p.clone();
    for i in indices {
        next.parts[i].position[axis] += target - coordinate(bounds(&p.parts[i]), source_mode);
    }
    next.validate()?;
    operations::protect_locked(p, &next)?;
    *p = next;
    Ok(())
}
/// Uniformly scale and rotate a selection around a world pivot, then translate.
/// Nodes remain independent, and group children retain local transforms.
pub fn transform_selection(
    p: &mut Project,
    ids: &Selection,
    pivot: [f64; 3],
    translation: [f64; 3],
    rotation: [f64; 3],
    scale: f64,
) -> Result<()> {
    let indices = selected(p, ids)?;
    ensure!(
        pivot
            .iter()
            .chain(&translation)
            .chain(&rotation)
            .all(|v| v.is_finite())
            && rotation.iter().all(|v| v.abs() <= 3600.)
            && scale.is_finite()
            && (0.00001..=100000.).contains(&scale),
        "Invalid selection transform"
    );
    let center = DVec3::from(pivot);
    let q = crate::document::quaternion(rotation);
    let mut next = p.clone();
    for i in indices {
        let n = &mut next.parts[i];
        n.position =
            (center + q * ((DVec3::from(n.position) - center) * scale) + DVec3::from(translation))
                .to_array();
        n.rotation = crate::document::rotation(q * crate::document::quaternion(n.rotation));
        if let Shape::Group { scale: s, .. } = &mut n.shape {
            *s *= scale;
        } else {
            let size = n.size().unwrap();
            for (axis, v) in size.iter().enumerate() {
                n.set_dimension(axis, v * scale);
            }
        }
    }
    next.validate()?;
    operations::protect_locked(p, &next)?;
    *p = next;
    Ok(())
}
/// Search includes nested component names, but selection remains at top level.
pub fn matches_query(n: &Node, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    n.name.to_lowercase().contains(&q)
        || n.kind().to_lowercase().contains(&q)
        || matches!(&n.shape,Shape::Group{children,..} if children.iter().any(|c|matches_query(c,&q)))
}
pub fn rename_selection(p: &mut Project, ids: &Selection, prefix: &str) -> Result<()> {
    let indices = selected(p, ids)?;
    let mut next = p.clone();
    let multiple = indices.len() > 1;
    for (order, i) in indices.into_iter().enumerate() {
        next.parts[i].name = if multiple {
            format!("{} {}", prefix.trim(), order + 1)
        } else {
            prefix.trim().to_string()
        };
    }
    ensure!(!prefix.trim().is_empty(), "Enter a name");
    next.validate()?;
    operations::protect_locked(p, &next)?;
    *p = next;
    Ok(())
}
