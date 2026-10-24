use std::io::{Cursor, Read};
use tinkercraft_core::{geometry, operations, three_mf, *};
fn multicolor() -> Project {
    let mut a = Node::primitive("Box");
    a.color = [220, 40, 50];
    a.position = [0., 0., 10.];
    let mut b = Node::primitive("Box");
    b.color = [30, 180, 100];
    b.position = [10., 0., 10.];
    let mut h = Node::primitive("Cylinder");
    h.hole = true;
    h.position = [5., 0., 10.];
    h.set_dimension(0, 6.);
    h.set_dimension(2, 30.);
    Project {
        name: "A & B <colored>".into(),
        parts: vec![a, b, h],
        ..Project::default()
    }
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 0.001, "{a} != {b}");
}
#[test]
fn colors_partition_boolean_volume_and_keep_group_cuts() {
    let mut p = multicolor();
    for grouped in [false, true] {
        if grouped {
            let ids = p.parts.iter().map(|n| n.id.clone()).collect();
            operations::group(&mut p, &ids).unwrap();
        }
        let parts = geometry::colored_parts(&p).unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].color, [220, 40, 50]);
        assert_eq!(parts[1].color, [30, 180, 100]);
        near(
            parts.iter().map(|p| p.mesh.volume()).sum(),
            evaluate(&p).unwrap().volume(),
        );
        let a = stl::manifold(&parts[0].mesh.vertices, &parts[0].mesh.triangles);
        let b = stl::manifold(&parts[1].mesh.vertices, &parts[1].mesh.triangles);
        near(geometry::mesh(&a.intersection(&b)).volume(), 0.);
    }
    p.parts[0].rotation = [15., 25., 35.];
    p.parts[0].position[0] += 40.;
    if let Shape::Group { scale, .. } = &mut p.parts[0].shape {
        *scale = 1.7;
    }
    near(
        geometry::colored_parts(&p)
            .unwrap()
            .iter()
            .map(|p| p.mesh.volume())
            .sum(),
        evaluate(&p).unwrap().volume(),
    );
}
#[test]
fn recolor_history_group_ungroup_and_legacy_defaults() {
    let mut p = multicolor();
    let original = p.clone();
    let ids = p.parts.iter().map(|n| n.id.clone()).collect();
    let group = operations::group(&mut p, &ids).unwrap();
    operations::ungroup(&mut p, &group).unwrap();
    assert_eq!(
        p.parts.iter().map(|n| n.color).collect::<Vec<_>>(),
        original.parts.iter().map(|n| n.color).collect::<Vec<_>>()
    );
    let mut history = History::new(p.clone(), true);
    p.parts[0].set_color_recursive([1, 2, 3]);
    history.commit(p).unwrap();
    history.undo();
    assert_eq!(history.project.parts[0].color, [220, 40, 50]);
    history.redo();
    assert_eq!(
        Project::parse(&history.project.bytes().unwrap()).unwrap(),
        history.project
    );
    let legacy = Project::parse(include_bytes!("../../../examples/plate.tinkercraft")).unwrap();
    assert_eq!(legacy.version, 8);
    assert_eq!(legacy.parts[0].color, document::default_color());
    let mut json = serde_json::to_value(&legacy).unwrap();
    json["parts"][0]["color"] = serde_json::json!([256, 0, 0]);
    assert!(Project::parse(&serde_json::to_vec(&json).unwrap()).is_err());
}
#[test]
fn three_mf_has_core_package_materials_assembly_and_escaped_names() {
    let p = multicolor();
    let bytes = three_mf::export(&p).unwrap();
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    assert_eq!(zip.len(), 3);
    assert!(zip.by_name("[Content_Types].xml").is_ok());
    assert!(zip.by_name("_rels/.rels").is_ok());
    let mut model = String::new();
    zip.by_name("3D/3dmodel.model")
        .unwrap()
        .read_to_string(&mut model)
        .unwrap();
    for expected in [
        "unit=\"millimeter\"",
        "#DC2832FF",
        "#1EB464FF",
        "A &amp; B &lt;colored&gt;",
        "<components>",
        "<build><item",
        "pid=\"1\" pindex=\"1\"",
    ] {
        assert!(model.contains(expected), "Missing {expected}");
    }
    let mut hidden = p.clone();
    hidden.parts[0].hidden = true;
    assert_eq!(
        three_mf::export(&hidden).unwrap(),
        three_mf::export(&p).unwrap()
    );
    assert!(three_mf::export(&Project::default()).is_err());
}

#[test]
fn export_names_filter_xml_forbidden_characters() {
    let mut p = multicolor();
    p.name = "A\u{ffff}B\u{fffe}C".into();
    let mut archive = zip::ZipArchive::new(Cursor::new(three_mf::export(&p).unwrap())).unwrap();
    let mut xml = String::new();
    archive
        .by_name("3D/3dmodel.model")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    assert!(xml.contains(">ABC</metadata>"));
    assert!(!xml.contains('\u{ffff}'));
}
