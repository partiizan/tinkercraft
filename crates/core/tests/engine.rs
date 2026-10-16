use glam::{DVec2, DVec3};
use tinkercraft_core::{
    geometry,
    operations::{self, Selection},
    storage, viewport, *,
};
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-5, "{a} != {b}");
}
fn group_demo() -> Project {
    let mut p = Project::demo();
    let s = p.parts.iter().map(|n| n.id.clone()).collect();
    operations::group(&mut p, &s).unwrap();
    p
}
#[test]
fn legacy_roundtrip() {
    let p = Project::demo();
    assert_eq!(p.version, 7);
    assert_eq!(p, Project::parse(&p.bytes().unwrap()).unwrap());
    let grouped = Project::parse(include_bytes!("../../../examples/group.tinkercraft")).unwrap();
    assert!(matches!(grouped.parts[0].shape, Shape::Group { .. }));
}
#[test]
fn malformed_documents() {
    let p = Project::demo();
    for bytes in [
        b"null".to_vec(),
        b"bad json".to_vec(),
        vec![b' '; 32_000_001],
    ] {
        assert!(Project::parse(&bytes).is_err());
    }
    let mut v: serde_json::Value = serde_json::from_slice(&p.bytes().unwrap()).unwrap();
    v["version"] = 8.into();
    assert!(Project::parse(&serde_json::to_vec(&v).unwrap()).is_err());
    v["version"] = 4.into();
    v["parts"][0].as_object_mut().unwrap().remove("rotation");
    assert!(Project::parse(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn limits_and_ids() {
    let mut p = Project::demo();
    p.parts.push(p.parts[0].clone());
    assert!(p.validate().is_err());
    p.parts.pop();
    p.parts[0].position[0] = 10001.;
    assert!(p.validate().is_err());
    p.parts[0].position[0] = f64::NAN;
    assert!(p.validate().is_err());
}
#[test]
fn plate_volume_bounds_stl() {
    let p = Project::demo();
    let m = evaluate(&p).unwrap();
    let area = 96. / 2. * 2.5f64.powi(2) * (std::f64::consts::TAU / 96.).sin();
    near(m.volume(), 12000. - 2. * area * 5.);
    let mut b = geometry::Bounds::default();
    for v in &m.vertices {
        b.add(DVec3::from(*v));
    }
    near(b.min.x, 0.);
    near(b.max.x, 80.);
    near(b.max.z, 5.);
    let bytes = m.stl().unwrap();
    assert_eq!(bytes.len(), 84 + 50 * m.triangles.len());
    assert_eq!(
        u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize,
        m.triangles.len()
    );
}
#[test]
fn empty_and_holes_save_but_do_not_export() {
    let mut p = Project::default();
    assert!(p.bytes().is_ok());
    assert!(evaluate(&p).is_err());
    p = Project::demo();
    for n in &mut p.parts {
        n.hole = true
    }
    assert!(p.bytes().is_ok());
    assert!(evaluate(&p).is_err());
}
#[test]
fn grouping_and_transform_ungroup() {
    let mut p = group_demo();
    let id = p.parts[0].id.clone();
    near(
        evaluate(&p).unwrap().volume(),
        evaluate(&Project::demo()).unwrap().volume(),
    );
    p.parts[0].rotation = [25., 30., 40.];
    if let Shape::Group { scale, .. } = &mut p.parts[0].shape {
        *scale = 1.5
    }
    let before = evaluate(&p).unwrap();
    operations::ungroup(&mut p, &id).unwrap();
    let after = evaluate(&p).unwrap();
    near(before.volume(), after.volume());
    for k in 0..3 {
        near(
            before
                .vertices
                .iter()
                .map(|p| p[k])
                .fold(f64::INFINITY, f64::min),
            after
                .vertices
                .iter()
                .map(|p| p[k])
                .fold(f64::INFINITY, f64::min),
        );
    }
}
#[test]
fn scoped_holes() {
    let mut p = group_demo();
    let mut box_node = Project::demo().parts[0].clone();
    box_node.id = "fill".into();
    p.parts.push(box_node);
    near(evaluate(&p).unwrap().volume(), 12000.);
}
#[test]
fn duplicate_recursive_ids() {
    let mut p = group_demo();
    let ids = Selection::from([p.parts[0].id.clone()]);
    operations::duplicate(&mut p, &ids);
    p.validate().unwrap();
    assert_eq!(p.parts.len(), 2);
}
#[test]
fn alignment_all_axes() {
    for axis in 0..3 {
        for mode in -1..=1 {
            let mut p = Project::demo();
            let ids = Selection::from(["left".into(), "right".into()]);
            let original = p.parts[0].clone();
            operations::align(&mut p, &ids, axis, mode).unwrap();
            assert_eq!(p.parts[0], original);
            let a = geometry::bounds(&p.parts[1]);
            let b = geometry::bounds(&p.parts[2]);
            near(
                if mode < 0 {
                    a.min[axis]
                } else if mode > 0 {
                    a.max[axis]
                } else {
                    a.center()[axis]
                },
                if mode < 0 {
                    b.min[axis]
                } else if mode > 0 {
                    b.max[axis]
                } else {
                    b.center()[axis]
                },
            );
        }
    }
}
#[test]
fn mirror_nested_cone_geometry() {
    for axis in 0..3 {
        let mut p = group_demo();
        let mut cone = Node::primitive("Cone");
        cone.position = [100., 20., 30.];
        cone.rotation = [15., 20., 30.];
        p.parts.push(cone);
        let ids = p.parts.iter().map(|n| n.id.clone()).collect();
        operations::group(&mut p, &ids).unwrap();
        p.parts[0].rotation = [12., 23., 34.];
        let id = p.parts[0].id.clone();
        let bounds = geometry::bounds(&p.parts[0]);
        let center = bounds.center()[axis];
        let before = evaluate(&p).unwrap();
        operations::mirror(&mut p, &Selection::from([id]), axis).unwrap();
        let after = evaluate(&p).unwrap();
        near(before.volume(), after.volume());
        for v in &before.vertices {
            let mut v = *v;
            v[axis] = 2. * center - v[axis];
            let dist = after
                .vertices
                .iter()
                .map(|w| DVec3::from(v).distance(DVec3::from(*w)))
                .fold(f64::INFINITY, f64::min);
            assert!(dist < 1e-5);
        }
    }
}
#[test]
fn sphere_cone_rotated_drop() {
    for kind in ["Sphere", "Cone"] {
        let mut p = Project::default();
        let mut n = Node::primitive(kind);
        n.rotation = [32., 41., 17.];
        p.parts.push(n);
        let ids = Selection::from([p.parts[0].id.clone()]);
        operations::drop_to_plane(&mut p, &ids).unwrap();
        let m = evaluate(&p).unwrap();
        near(
            m.vertices
                .iter()
                .map(|v| v[2])
                .fold(f64::INFINITY, f64::min),
            0.,
        );
        let expected = if kind == "Sphere" {
            4. / 3. * std::f64::consts::PI * 1000.
        } else {
            std::f64::consts::PI * 100. * 20. / 3.
        };
        assert!((m.volume() / expected - 1.).abs() < 0.01);
    }
}
#[test]
fn history_baseline_and_branching() {
    let p = Project::demo();
    let mut h = History::new(p.clone(), true);
    let mut next = p.clone();
    next.parts[0].position[0] += 5.;
    h.commit(next.clone()).unwrap();
    assert!(h.dirty());
    h.mark_saved();
    assert!(!h.dirty());
    h.undo();
    assert!(h.dirty());
    assert_eq!(h.project, p);
    h.redo();
    assert_eq!(h.project, next);
    assert!(!h.dirty());
    h.undo();
    let mut branch = h.project.clone();
    branch.name = "Branch".into();
    h.commit(branch).unwrap();
    assert!(!h.can_redo());
}
#[test]
fn atomic_file_external_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("model.tinkercraft");
    let mut file = storage::DocumentFile::default();
    file.save(&Project::demo(), Some(path.clone())).unwrap();
    let (_, p) = storage::DocumentFile::open(path.clone()).unwrap();
    assert_eq!(p, Project::demo());
    std::fs::write(&path, b"outside change").unwrap();
    assert!(file.save(&p, None).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"outside change");
}
#[test]
fn projection_and_picking() {
    let p = Project::demo();
    let sources = geometry::sources(&p);
    let mut camera = viewport::Camera::default();
    camera.view("Top");
    camera.orthographic = true;
    let a = camera.project(DVec3::new(40., 15., 0.), 320, 240).unwrap();
    let b = camera.project(DVec3::new(40., 15., 20.), 320, 240).unwrap();
    near(a[0], b[0]);
    near(a[1], b[1]);
    let (o, d) = camera.ray(DVec2::new(160., 120.), 320., 240.);
    assert!(d.z < -0.99 && o.z > 0.);
    let frame = viewport::render(&camera, 320, 240, &sources, &Selection::new(), None);
    assert_eq!(frame.pick(160, 120).as_deref(), Some("plate"));
    assert_eq!(frame.rgba.len(), 320 * 240 * 4);
}
#[test]
fn checkpoint6_cross_engine_compatibility() {
    let fixtures: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../examples/compatibility-fixtures.json"
    ))
    .unwrap();
    for fixture in fixtures.as_array().unwrap() {
        let p = Project::parse(&serde_json::to_vec(&fixture["project"]).unwrap()).unwrap();
        let mesh = evaluate(&p).unwrap();
        let expected = fixture["volume"].as_f64().unwrap();
        assert!(
            (mesh.volume() - expected).abs() < expected.abs() * 1e-6,
            "{} volume",
            p.name
        );
        for axis in 0..3 {
            let min = mesh
                .vertices
                .iter()
                .map(|p| p[axis])
                .fold(f64::INFINITY, f64::min);
            let max = mesh
                .vertices
                .iter()
                .map(|p| p[axis])
                .fold(f64::NEG_INFINITY, f64::max);
            near(min, fixture["min"][axis].as_f64().unwrap());
            near(max, fixture["max"][axis].as_f64().unwrap());
        }
    }
}
