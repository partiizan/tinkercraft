use glam::DVec3;
use tinkercraft_core::{geometry, operations, viewport, *};

fn grouped_plate() -> Project {
    let mut p = Project::demo();
    let ids = p.parts.iter().map(|n| n.id.clone()).collect();
    operations::group(&mut p, &ids).unwrap();
    p
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 0.001, "{a} != {b}");
}
#[test]
fn cut_group_is_pickable_but_its_holes_are_not() {
    let p = grouped_plate();
    let sources = geometry::editable_sources(&p).unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].id, p.parts[0].id);
    near(sources[0].mesh.volume(), evaluate(&p).unwrap().volume());
    let mut camera = viewport::Camera::default();
    camera.view("Top");
    camera.orthographic = true;
    let frame = viewport::render(&camera, 600, 600, &sources, &Default::default(), None);
    let pick = |point| {
        let q = camera.project(DVec3::from(point), 600, 600).unwrap();
        frame.pick(q[0] as usize, q[1] as usize)
    };
    assert_eq!(pick([40., 15., 5.]), Some(p.parts[0].id.clone()));
    for hole in Project::demo().parts.iter().filter(|n| n.hole) {
        assert_eq!(pick(hole.position), None);
    }
}
#[test]
fn transformed_resized_group_keeps_cuts_and_identity() {
    let mut p = grouped_plate();
    let before = p.parts[0].clone();
    let old = geometry::editable_sources(&p).unwrap().remove(0);
    let width = operations::local_dimensions(&before)[0];
    let mut changed = operations::resize_node(&before, 0, width * 1.5).unwrap();
    changed.position[0] += 20.;
    changed.rotation = [15., 25., 35.];
    let preview = old
        .mesh
        .transformed(operations::resize_preview_matrix(&before, &changed));
    p.parts[0] = changed;
    let sources = geometry::editable_sources(&p).unwrap();
    assert_eq!(sources[0].id, old.id);
    near(sources[0].mesh.volume(), old.mesh.volume() * 1.5f64.powi(3));
    near(sources[0].mesh.volume(), evaluate(&p).unwrap().volume());
    near(preview.volume(), sources[0].mesh.volume());
    for v in preview.vertices {
        assert!(
            sources[0]
                .mesh
                .vertices
                .iter()
                .any(|w| DVec3::from(v).distance(DVec3::from(*w)) < 0.001)
        );
    }
}
#[test]
fn group_accepts_more_holes_and_survives_undo_and_reload() {
    let p = grouped_plate();
    let volume = evaluate(&p).unwrap().volume();
    let mut h = History::new(p.clone(), true);
    let mut next = p.clone();
    let mut hole = Node::primitive("Cylinder");
    hole.hole = true;
    hole.position = [40., 15., 2.5];
    hole.set_dimension(0, 4.);
    hole.set_dimension(2, 20.);
    next.parts.push(hole);
    let ids = next.parts.iter().map(|n| n.id.clone()).collect();
    operations::group(&mut next, &ids).unwrap();
    let sources = geometry::editable_sources(&next).unwrap();
    assert_eq!(sources.len(), 1);
    assert!(sources[0].mesh.volume() < volume);
    near(sources[0].mesh.volume(), evaluate(&next).unwrap().volume());
    h.commit(next.clone()).unwrap();
    h.undo();
    assert_eq!(h.project, p);
    h.redo();
    let loaded = Project::parse(&h.project.bytes().unwrap()).unwrap();
    near(
        geometry::editable_sources(&loaded).unwrap()[0]
            .mesh
            .volume(),
        sources[0].mesh.volume(),
    );
    next.parts[0].hidden = true;
    assert!(geometry::editable_sources(&next).unwrap().is_empty());
}
