use glam::DVec3;
use tinkercraft_core::{
    geometry,
    operations::{self, Selection},
    *,
};
fn p(n: Node) -> Project {
    Project {
        parts: vec![n],
        ..Project::default()
    }
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-5, "{a} != {b}");
}
#[test]
fn wedge_roof_volume_bounds_and_roundtrip() {
    for kind in ["Wedge", "Roof"] {
        let mut n = Node::primitive(kind);
        n.set_dimension(0, 40.);
        n.set_dimension(1, 30.);
        n.set_dimension(2, 10.);
        let b = geometry::bounds(&n);
        near((b.max - b.min).x, 40.);
        near((b.max - b.min).y, 30.);
        near((b.max - b.min).z, 10.);
        let p = p(n);
        near(evaluate(&p).unwrap().volume(), 6000.);
        assert_eq!(p, Project::parse(&p.bytes().unwrap()).unwrap());
        let stl = evaluate(&p).unwrap().stl().unwrap();
        let reimport = stl::import(&stl, kind, 1.).unwrap();
        near(evaluate(&super_project(reimport)).unwrap().volume(), 6000.);
    }
}
fn super_project(n: Node) -> Project {
    p(n)
}
#[test]
fn asymmetric_shapes_mirror_on_every_axis() {
    for kind in ["Wedge", "Roof"] {
        for axis in 0..3 {
            let mut n = Node::primitive(kind);
            n.rotation = [12., 23., 34.];
            n.set_dimension(0, 31.);
            n.set_dimension(1, 17.);
            let center = geometry::bounds(&n).center()[axis];
            let id = n.id.clone();
            let mut project = p(n);
            let before = evaluate(&project).unwrap();
            operations::mirror(&mut project, &Selection::from([id]), axis).unwrap();
            let after = evaluate(&project).unwrap();
            near(before.volume(), after.volume());
            for mut v in before.vertices {
                v[axis] = 2. * center - v[axis];
                assert!(
                    after
                        .vertices
                        .iter()
                        .any(|w| DVec3::from(v).distance(DVec3::from(*w)) < 1e-5)
                );
            }
        }
    }
}
#[test]
fn resize_constraints_and_preview_match_committed_geometry() {
    for kind in ["Box", "Wedge", "Roof", "Sphere", "Cylinder", "Cone"] {
        for axis in 0..3 {
            let mut before = Node::primitive(kind);
            before.rotation = [23., 34., 45.];
            let after = operations::resize_node(&before, axis, 32.).unwrap();
            assert_eq!(before.position, after.position);
            near(after.size().unwrap()[axis], 32.);
            if matches!(after.shape, Shape::Sphere { .. }) {
                assert_eq!(after.size().unwrap(), [32.; 3]);
            }
            if matches!(after.shape, Shape::Cylinder { .. } | Shape::Cone { .. }) {
                let s = after.size().unwrap();
                near(s[0], s[1]);
            }
            let matrix = operations::resize_preview_matrix(&before, &after);
            let source = geometry::sources(&p(before));
            let target = geometry::sources(&p(after));
            let preview = source[0].mesh.transformed(matrix);
            // Sphere source coordinates retain legacy f32 tessellation rounding.
            assert!((preview.volume() / target[0].mesh.volume() - 1.).abs() < 1e-7);
            for v in preview.vertices {
                assert!(
                    target[0]
                        .mesh
                        .vertices
                        .iter()
                        .any(|w| DVec3::from(v).distance(DVec3::from(*w)) < 1e-4)
                );
            }
        }
    }
}
#[test]
fn group_resize_uniform_undo_redo_and_bounds() {
    let mut project = Project::demo();
    let ids = project.parts.iter().map(|n| n.id.clone()).collect();
    operations::group(&mut project, &ids).unwrap();
    project.parts[0].rotation = [10., 20., 30.];
    let before = project.clone();
    let dims = operations::local_dimensions(&project.parts[0]);
    project.parts[0] = operations::resize_node(&project.parts[0], 0, dims[0] * 2.).unwrap();
    near(
        evaluate(&project).unwrap().volume(),
        evaluate(&before).unwrap().volume() * 8.,
    );
    let nextdims = operations::local_dimensions(&project.parts[0]);
    for i in 0..3 {
        near(nextdims[i], dims[i] * 2.);
    }
    let mut h = History::new(before.clone(), true);
    h.commit(project.clone()).unwrap();
    h.undo();
    assert_eq!(h.project, before);
    h.redo();
    assert_eq!(h.project, project);
}
#[test]
fn invalid_resize_preserves_input() {
    let n = Node::primitive("Box");
    let original = n.clone();
    for dimension in [0., -1., f64::NAN, 1001.] {
        assert!(operations::resize_node(&n, 0, dimension).is_err());
    }
    assert!(operations::resize_node(&n, 3, 20.).is_err());
    assert_eq!(n, original);
}
#[test]
fn wedge_hole_group_and_drop() {
    for kind in ["Wedge", "Roof"] {
        let mut project = p(Node::primitive("Box"));
        let mut hole = Node::primitive(kind);
        hole.hole = true;
        project.parts.push(hole);
        near(evaluate(&project).unwrap().volume(), 4000.);
        let ids = project.parts.iter().map(|n| n.id.clone()).collect();
        let id = operations::group(&mut project, &ids).unwrap();
        near(evaluate(&project).unwrap().volume(), 4000.);
        operations::ungroup(&mut project, &id).unwrap();
        near(evaluate(&project).unwrap().volume(), 4000.);
        project.parts.pop();
        project.parts[0] = Node::primitive(kind);
        project.parts[0].rotation = [36., 24., 17.];
        let ids = Selection::from([project.parts[0].id.clone()]);
        operations::drop_to_plane(&mut project, &ids).unwrap();
        near(geometry::bounds(&project.parts[0]).min.z, 0.);
    }
}
