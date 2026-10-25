use glam::DVec3;
use tinkercraft_core::{
    geometry,
    history::History,
    modeling::{self, Repeat},
    operations::{self, Selection},
    *,
};
fn ids(p: &Project) -> Selection {
    p.parts.iter().map(|n| n.id.clone()).collect()
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-6, "{a} != {b}");
}
fn three() -> Project {
    let mut p = Project::default();
    for (x, width) in [(0., 10.), (17., 20.), (80., 30.)] {
        let mut n = Node::primitive("Box");
        n.position[0] = x;
        n.set_dimension(0, width);
        p.parts.push(n);
    }
    p
}
#[test]
fn distribution_spacing_anchors_and_atomic_rejection() {
    for gaps in [false, true] {
        let mut p = three();
        let before = p.clone();
        let s = ids(&p);
        modeling::distribute(&mut p, &s, 0, gaps).unwrap();
        assert_eq!(p.parts[0], before.parts[0]);
        assert_eq!(p.parts[2], before.parts[2]);
        let b: Vec<_> = p.parts.iter().map(geometry::bounds).collect();
        if gaps {
            near(b[1].min.x - b[0].max.x, b[2].min.x - b[1].max.x)
        } else {
            near(
                b[1].center().x - b[0].center().x,
                b[2].center().x - b[1].center().x,
            )
        }
        let mut h = History::new(before.clone(), true);
        h.commit(p.clone()).unwrap();
        assert!(h.undo());
        assert_eq!(h.project, before);
        assert!(h.redo());
        assert_eq!(h.project, p);
        p.parts[1].locked = true;
        let before = p.clone();
        assert!(modeling::distribute(&mut p, &s, 0, gaps).is_err());
        assert_eq!(p, before);
    }
}
#[test]
fn repeated_groups_keep_cuts_and_get_unique_nested_ids() {
    let mut p = Project::demo();
    let s = ids(&p);
    operations::group(&mut p, &s).unwrap();
    let s = ids(&p);
    let volume = evaluate(&p).unwrap().volume();
    let original = p.parts[0].clone();
    let added = modeling::repeat(&mut p, &s, 2, Repeat::Linear([100., 0., 0.])).unwrap();
    assert_eq!(added.len(), 2);
    p.validate().unwrap();
    near(evaluate(&p).unwrap().volume(), volume * 3.);
    for (i, n) in p.parts.iter().enumerate() {
        near(n.position[0], original.position[0] + i as f64 * 100.);
        assert_eq!(n.color, original.color);
        if let Shape::Group { children, .. } = &n.shape {
            assert_eq!(children.len(), 3);
            assert_eq!(children.iter().filter(|n| n.hole).count(), 2)
        } else {
            panic!()
        }
    }
    assert_eq!(p, Project::parse(&p.bytes().unwrap()).unwrap());
    let before = p.clone();
    assert!(modeling::repeat(&mut p, &s, 99, Repeat::Linear([1., 0., 0.])).is_err());
    assert_eq!(before, p);
}
#[test]
fn radial_full_circle_has_no_duplicate_endpoint() {
    let mut p = Project::default();
    let mut n = Node::primitive("Box");
    n.position = [10., 0., 0.];
    p.parts.push(n);
    let s = ids(&p);
    modeling::repeat(
        &mut p,
        &s,
        3,
        Repeat::Radial {
            pivot: [0.; 3],
            degrees: 360.,
            rotate: true,
        },
    )
    .unwrap();
    for (n, expected) in p
        .parts
        .iter()
        .zip([[10., 0.], [0., 10.], [-10., 0.], [0., -10.]])
    {
        near(n.position[0], expected[0]);
        near(n.position[1], expected[1]);
    }
    let before = p.clone();
    assert!(
        modeling::repeat(
            &mut p,
            &s,
            1,
            Repeat::Radial {
                pivot: [0.; 3],
                degrees: 0.,
                rotate: false
            }
        )
        .is_err()
    );
    assert_eq!(before, p);
}
#[test]
fn child_hole_edit_preserves_group_and_undo() {
    let mut p = Project::demo();
    let s = ids(&p);
    operations::group(&mut p, &s).unwrap();
    p.parts[0].rotation = [0., 0., 30.];
    let original = p.parts[0].clone();
    let before = p.clone();
    let volume = evaluate(&p).unwrap().volume();
    let mut edited = original.clone();
    if let Shape::Group { children, .. } = &mut edited.shape {
        let hole = children.iter_mut().find(|n| n.hole).unwrap();
        hole.set_dimension(0, 10.);
        hole.position[0] += 2.;
    }
    modeling::edit_group(&mut p, &original, edited.clone()).unwrap();
    assert!(evaluate(&p).unwrap().volume() < volume);
    assert_eq!(p.parts[0].rotation, original.rotation);
    assert_eq!(p.parts[0].id, original.id);
    let changed = p.clone();
    assert!(modeling::edit_group(&mut p, &original, edited).is_err());
    assert_eq!(p, changed);
    let mut h = History::new(before.clone(), true);
    h.commit(p.clone()).unwrap();
    h.undo();
    assert_eq!(h.project, before);
    h.redo();
    assert_eq!(h.project, p);
}
#[test]
fn resolution_changes_mesh_and_preserves_legacy() {
    for kind in ["Cylinder", "Cone", "Sphere"] {
        let mut p = Project::default();
        p.parts.push(Node::primitive(kind));
        p.parts[0].segments = 24;
        let low = evaluate(&p).unwrap();
        p.parts[0].segments = 96;
        let high = evaluate(&p).unwrap();
        assert!(high.triangles.len() > low.triangles.len());
        assert!(high.stl().is_ok());
        p.parts[0].segments = 192;
        assert!(evaluate(&p).unwrap().triangles.len() > high.triangles.len());
        assert_eq!(p, Project::parse(&p.bytes().unwrap()).unwrap());
        p.parts[0].segments = 25;
        assert!(p.validate().is_err());
    }
    let mut legacy = serde_json::to_value(Project::demo()).unwrap();
    legacy["version"] = 8.into();
    for n in legacy["parts"].as_array_mut().unwrap() {
        n.as_object_mut().unwrap().remove("segments");
    }
    let p = Project::parse(&serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert_eq!(p.version, 9);
    assert!(p.parts.iter().all(|n| n.segments == 96));
    near(
        evaluate(&p).unwrap().volume(),
        evaluate(&Project::demo()).unwrap().volume(),
    );
}
#[test]
fn ruler_workplane_snap_and_invalid_rays() {
    let p = modeling::workplane_point(DVec3::new(1.23, 2.76, 10.), -DVec3::Z, 0.5).unwrap();
    assert_eq!(p, DVec3::new(1., 3., 0.));
    assert!(modeling::workplane_point(DVec3::Z, DVec3::X, 1.).is_none());
    assert!(modeling::workplane_point(DVec3::Z, DVec3::Z, 1.).is_none());
    let p = three();
    let a = geometry::bounds(&p.parts[0]);
    let b = geometry::bounds(&p.parts[2]);
    let g = modeling::bounds_gap(&a, &b);
    near(g.x, b.min.x - a.max.x);
    near(g.y, 0.);
}
