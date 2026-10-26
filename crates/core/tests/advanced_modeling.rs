use glam::DVec3;
use tinkercraft_core::{
    geometry,
    history::History,
    modeling,
    operations::{self, Selection},
    *,
};
fn one(kind: &str) -> Project {
    Project {
        parts: vec![Node::primitive(kind)],
        ..Project::default()
    }
}
fn ids(p: &Project) -> Selection {
    p.parts.iter().map(|n| n.id.clone()).collect()
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-5, "{a} != {b}");
}
#[test]
fn tube_and_torus_are_hollow_closed_exportable_solids() {
    for kind in ["Tube", "Torus"] {
        let p = one(kind);
        let n = &p.parts[0];
        let solid = geometry::primitive(n);
        let m = evaluate(&p).unwrap();
        assert!(m.volume() > 0.);
        assert!(m.stl().is_ok());
        tinkercraft_core::stl::import(&m.stl().unwrap(), "roundtrip", 1.).unwrap();
        let mut probe = Node::primitive("Box");
        probe.position = [0.; 3];
        for a in 0..3 {
            probe.set_dimension(a, 1.);
        }
        near(
            geometry::mesh(&solid.intersection(&geometry::primitive(&probe))).volume(),
            0.,
        );
        if kind == "Tube" {
            let area = 96. / 2. * (std::f64::consts::TAU / 96.).sin() * (100. - 36.);
            near(m.volume(), area * 20.);
        } else {
            let analytic = 2. * std::f64::consts::PI.powi(2) * 7. * 3. * 3.;
            assert!((m.volume() / analytic - 1.).abs() < 0.02);
        }
        let bounds = geometry::bounds(n);
        near(bounds.max.z - bounds.min.z, n.size().unwrap()[2]);
        assert_eq!(p, Project::parse(&p.bytes().unwrap()).unwrap());
    }
}
#[test]
fn polygon_sides_and_mirror_preserve_actual_geometry() {
    for sides in [3, 5, 6, 32] {
        let mut p = one("Polygon");
        if let Shape::Polygon { sides: s, .. } = &mut p.parts[0].shape {
            *s = sides;
        }
        let expected =
            sides as f64 / 2. * 100. * (std::f64::consts::TAU / sides as f64).sin() * 20.;
        near(evaluate(&p).unwrap().volume(), expected);
        p.parts[0].rotation = [15., 25., 35.];
        let before = geometry::points(&p.parts[0]);
        let matrix = p.parts[0].matrix();
        let s = ids(&p);
        let plane = geometry::bounds(&p.parts[0]).center().x;
        operations::mirror(&mut p, &s, 0).unwrap();
        let after: Vec<_> = geometry::points(&p.parts[0])
            .iter()
            .map(|v| p.parts[0].matrix().transform_point3(DVec3::from(*v)))
            .collect();
        for v in before {
            let mut world = matrix.transform_point3(DVec3::from(v));
            world.x = 2. * plane - world.x;
            assert!(after.iter().any(|v| v.distance(world) < 1e-6));
        }
    }
}
#[test]
fn block_text_holes_editing_mirror_and_roundtrip() {
    let mut p = one("Text");
    if let Shape::Text { content, .. } = &mut p.parts[0].shape {
        *content = "O".into();
    }
    let n = &p.parts[0];
    let m = evaluate(&p).unwrap();
    assert!(m.stl().is_ok());
    let mut probe = Node::primitive("Box");
    for a in 0..3 {
        probe.set_dimension(a, 0.1);
    }
    near(
        geometry::mesh(&geometry::primitive(n).intersection(&geometry::primitive(&probe))).volume(),
        0.,
    );
    if let Shape::Text { content, .. } = &mut p.parts[0].shape {
        *content = "LABEL 12".into();
    }
    let before = p.clone();
    let s = ids(&p);
    operations::mirror(&mut p, &s, 0).unwrap();
    assert_eq!(p, Project::parse(&p.bytes().unwrap()).unwrap());
    operations::mirror(&mut p, &s, 0).unwrap();
    assert_eq!(p, before);
    let text = p.parts[0].clone();
    let mut plate = Node::primitive("Box");
    plate.shape = Shape::Box {
        size: [60., 20., 3.],
    };
    plate.position = text.position;
    p.parts.insert(0, plate);
    p.parts[1].hole = true;
    let full = 60. * 20. * 3.;
    let cut = evaluate(&p).unwrap().volume();
    assert!(cut < full && cut > 0.);
    let s = ids(&p);
    operations::group(&mut p, &s).unwrap();
    near(evaluate(&p).unwrap().volume(), cut);
    let original = p.parts[0].clone();
    let mut edited = original.clone();
    if let Shape::Group { children, .. } = &mut edited.shape {
        if let Shape::Text { content, .. } = &mut children[1].shape {
            *content = "AB".into();
        } else {
            panic!()
        }
    }
    modeling::edit_group(&mut p, &original, edited).unwrap();
    assert!((evaluate(&p).unwrap().volume() - cut).abs() > 0.01);
}
#[test]
fn alphabet_is_printable_and_invalid_inputs_rejected() {
    for c in "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._+".chars() {
        let mut p = one("Text");
        if let Shape::Text { content, .. } = &mut p.parts[0].shape {
            *content = c.to_string();
        }
        let mesh = evaluate(&p).unwrap();
        assert!(mesh.volume() > 0.);
        assert!(mesh.stl().is_ok(), "{c}");
        tinkercraft_core::stl::import(&mesh.stl().unwrap(), "letter", 1.).unwrap();
    }
    for invalid in ["", "   ", "café", "Hello!", "12345678901234567890123456"] {
        assert!(tinkercraft_core::catalog::validate_text(invalid).is_err());
    }
    for kind in ["Tube", "Torus"] {
        let mut p = one(kind);
        match &mut p.parts[0].shape {
            Shape::Tube { bore, .. } | Shape::Torus { bore, .. } => *bore = 1.,
            _ => unreachable!(),
        };
        assert!(p.validate().is_err());
    }
    let mut legacy = serde_json::to_value(one("Tube")).unwrap();
    legacy["version"] = 9.into();
    assert!(Project::parse(&serde_json::to_vec(&legacy).unwrap()).is_err());
}
#[test]
fn reference_alignment_leaves_locked_reference_fixed() {
    let mut p = Project::demo();
    p.parts[0].locked = true;
    let reference = p.parts[0].clone();
    let s = ids(&p);
    modeling::align_reference(&mut p, &s, &reference.id, 0, -1, 1, 2.).unwrap();
    assert_eq!(p.parts[0], reference);
    for n in &p.parts[1..] {
        near(
            geometry::bounds(n).min.x,
            geometry::bounds(&reference).max.x + 2.,
        );
    }
    let before = p.clone();
    p.parts[1].locked = true;
    let locked = p.clone();
    assert!(modeling::align_reference(&mut p, &s, &reference.id, 1, 0, 0, 0.).is_err());
    assert_eq!(p, locked);
    p = before.clone();
    assert!(modeling::align_reference(&mut p, &s, &reference.id, 3, 0, 0, 0.).is_err());
    assert_eq!(p, before);
}
#[test]
fn multi_transform_preserves_independence_and_group_cuts() {
    let mut p = Project::demo();
    let s = ids(&p);
    operations::group(&mut p, &s).unwrap();
    let mut n = Node::primitive("Tube");
    n.position = [140., 15., 10.];
    p.parts.push(n);
    let before = p.clone();
    let s = ids(&p);
    let v = evaluate(&p).unwrap().volume();
    modeling::transform_selection(&mut p, &s, [0.; 3], [1., 2., 3.], [0., 0., 90.], 2.).unwrap();
    assert_eq!(p.parts.len(), 2);
    assert_eq!(ids(&p), s);
    for (a, b) in before.parts.iter().zip(&p.parts) {
        near(b.position[0], -2. * a.position[1] + 1.);
        near(b.position[1], 2. * a.position[0] + 2.);
        near(b.position[2], 2. * a.position[2] + 3.);
    }
    near(evaluate(&p).unwrap().volume(), v * 8.);
    let mut h = History::new(before.clone(), true);
    h.commit(p.clone()).unwrap();
    h.undo();
    assert_eq!(h.project, before);
    h.redo();
    assert_eq!(h.project, p);
    let original = p.clone();
    assert!(modeling::transform_selection(&mut p, &s, [0.; 3], [0.; 3], [0.; 3], 100000.).is_err());
    assert_eq!(p, original);
}
#[test]
fn browser_nested_search_and_atomic_renaming() {
    let mut p = Project::demo();
    let s = ids(&p);
    modeling::rename_selection(&mut p, &s, "Mount").unwrap();
    assert_eq!(p.parts[0].name, "Mount 1");
    assert_eq!(p.parts[2].name, "Mount 3");
    let before = p.clone();
    assert!(modeling::rename_selection(&mut p, &s, " ").is_err());
    assert_eq!(p, before);
    operations::group(&mut p, &s).unwrap();
    assert!(modeling::matches_query(&p.parts[0], "MOUNT 3"));
    assert!(modeling::matches_query(&p.parts[0], "cylinder"));
    assert!(!modeling::matches_query(&p.parts[0], "no-match"));
}

#[test]
fn bundled_catalog_roundtrips_and_exports() {
    let p = Project::parse(include_bytes!(
        "../../../examples/modeling-catalog.tinkercraft"
    ))
    .unwrap();
    let m = evaluate(&p).unwrap();
    assert!(m.volume() > 0.);
    tinkercraft_core::stl::import(&m.stl().unwrap(), "catalog", 1.).unwrap();
    assert_eq!(p, Project::parse(&p.bytes().unwrap()).unwrap());
}
