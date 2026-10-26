use glam::{DVec2, DVec3};
use tinkercraft_core::{
    extrusion, geometry,
    operations::{self, Selection},
    spatial::{self, Workplane},
    *,
};
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-5, "{a} != {b}")
}
#[test]
fn svg_cutouts_transforms_strokes_and_roundtrip() {
    let svg=br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g transform="translate(10 20)"><path fill-rule="evenodd" d="M0 0H40V20H0Z M10 5H30V15H10Z"/></g></svg>"#;
    let n = extrusion::svg(svg, 40., 3., "SVG").unwrap();
    let p = Project {
        parts: vec![n],
        ..Project::default()
    };
    near(evaluate(&p).unwrap().volume(), 1800.);
    assert!(
        p == Project::parse(&p.bytes().unwrap()).unwrap(),
        "Exact geometry save/reopen roundtrip"
    );
    stl::import(&evaluate(&p).unwrap().stl().unwrap(), "roundtrip", 1.).unwrap();
    let stroke=br#"<svg xmlns="http://www.w3.org/2000/svg"><path fill="none" stroke="black" stroke-width="2" stroke-linecap="round" d="M0 0 L20 0"/></svg>"#;
    let n = extrusion::svg(stroke, 22., 2., "Stroke").unwrap();
    let p = Project {
        parts: vec![n],
        ..Project::default()
    };
    assert!(evaluate(&p).unwrap().volume() > 80.);
    for bad in [
        br#"<svg><image href="file:///etc/passwd"/></svg>"#.as_slice(),
        br#"<svg><text>Hello</text></svg>"#,
        br#"<svg><clipPath/></svg>"#,
        br#"<!DOCTYPE svg [<!ENTITY x SYSTEM "file:///etc/passwd">]><svg>&x;</svg>"#,
    ] {
        assert!(extrusion::svg(bad, 40., 2., "bad").is_err());
    }
}
#[test]
fn font_outlines_are_self_contained_printable_and_bounded() {
    for font in [
        epaint_default_fonts::UBUNTU_LIGHT,
        epaint_default_fonts::HACK_REGULAR,
    ] {
        let n = extrusion::font(font, "BOLT 12", 60., 2.).unwrap();
        let p = Project {
            parts: vec![n],
            ..Project::default()
        };
        assert!(evaluate(&p).unwrap().volume() > 1.);
        assert!(
            p == Project::parse(&p.bytes().unwrap()).unwrap(),
            "Exact geometry save/reopen roundtrip"
        );
        stl::import(&evaluate(&p).unwrap().stl().unwrap(), "letters", 1.).unwrap();
    }
    assert!(extrusion::font(&[], "Test", 40., 2.).is_err());
    assert!(extrusion::font(epaint_default_fonts::UBUNTU_LIGHT, "مرحبا", 40., 2.).is_err());
    assert!(extrusion::font(epaint_default_fonts::UBUNTU_LIGHT, " ", 40., 2.).is_err());
}
#[test]
fn tilted_plane_roundtrip_picking_and_drop() {
    let plane = Workplane {
        origin: [4., 5., 6.],
        rotation: [30., 20., 10.],
    };
    let local = DVec3::new(2.24, 3.76, 0.);
    let point = plane.world(local);
    let hit = plane
        .intersect(point + plane.normal() * 10., -plane.normal(), 0.5)
        .unwrap();
    assert!(plane.local(hit).distance(DVec3::new(2., 4., 0.)) < 1e-6);
    let mut n = Node::primitive("Box");
    n.rotation = plane.rotation;
    n.position = plane.world(DVec3::new(0., 0., 40.)).to_array();
    let mut p = Project {
        parts: vec![n],
        workplane: plane.clone(),
        ..Project::default()
    };
    let s = Selection::from([p.parts[0].id.clone()]);
    spatial::drop_to_workplane(&mut p, &s).unwrap();
    let min = geometry::sources(&p)
        .iter()
        .flat_map(|s| s.mesh.vertices.iter())
        .map(|v| plane.local(DVec3::from(*v)).z)
        .fold(f64::INFINITY, f64::min);
    near(min, 0.);
    assert!(
        p == Project::parse(&p.bytes().unwrap()).unwrap(),
        "Exact geometry save/reopen roundtrip"
    );
    let reconstructed = Workplane::on_face(point, plane.normal()).unwrap();
    assert!(reconstructed.normal().distance(plane.normal()) < 1e-6);
    p.parts[0].hole = true;
    spatial::drop_to_workplane(&mut p, &s).unwrap();
}
#[test]
fn ray_pick_returns_closest_surface_and_snapping_vertex() {
    let mut n = Node::primitive("Box");
    n.position = [0.; 3];
    let p = Project {
        parts: vec![n],
        ..Project::default()
    };
    let sources = geometry::sources(&p);
    let hit = spatial::pick(&sources, DVec3::new(1., 2., 40.), -DVec3::Z).unwrap();
    near(hit.point.z, 10.);
    assert!(hit.normal.z > 0.99);
    assert_eq!(hit.id, p.parts[0].id);
    assert!(hit.vertices.contains(&spatial::snap_point(&hit, true)));
    assert!(spatial::pick(&sources, DVec3::new(100., 100., 40.), -DVec3::Z).is_none());
}
#[test]
fn group_replacement_preserves_parent_and_full_validation() {
    let mut p = Project::demo();
    let s = p.parts.iter().map(|n| n.id.clone()).collect();
    operations::group(&mut p, &s).unwrap();
    p.parts[0].rotation = [10., 20., 30.];
    let original = p.clone();
    let Shape::Group { children, .. } = &p.parts[0].shape else {
        panic!()
    };
    let mut children = children.clone();
    children[1].set_dimension(0, 10.);
    let edited = spatial::replace_children(&p, &p.parts[0].id, children).unwrap();
    assert_eq!(edited.parts[0].position, p.parts[0].position);
    assert_eq!(edited.parts[0].rotation, p.parts[0].rotation);
    assert!(evaluate(&edited).unwrap().volume() < evaluate(&p).unwrap().volume());
    let mut h = History::new(p, true);
    h.commit(edited.clone()).unwrap();
    h.undo();
    assert_eq!(h.project, original);
    h.redo();
    assert_eq!(h.project, edited);
    assert!(spatial::replace_children(&edited, &edited.parts[0].id, vec![]).is_err());
}
#[test]
fn marquee_excludes_hidden_and_locked() {
    let mut p = Project::demo();
    let c = viewport::Camera::default();
    let all = spatial::box_select(&p, &c, 900, 700, DVec2::ZERO, DVec2::new(900., 700.));
    assert_eq!(all.len(), 3);
    p.parts[0].locked = true;
    p.parts[1].hidden = true;
    assert_eq!(
        spatial::box_select(&p, &c, 900, 700, DVec2::ZERO, DVec2::new(900., 700.)).len(),
        1
    );
}
