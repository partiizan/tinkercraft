use tinkercraft_core::{
    geometry,
    operations::{self, Selection},
    stl, *,
};
fn cube() -> Mesh {
    geometry::mesh(&geometry::primitive(&Node::primitive("Box")))
}
fn project(n: Node) -> Project {
    Project {
        parts: vec![n],
        ..Project::default()
    }
}
fn ascii(m: &Mesh) -> Vec<u8> {
    let mut s = String::from("solid test\n");
    for t in &m.triangles {
        s.push_str("facet normal 0 0 0\nouter loop\n");
        for i in t {
            let [x, y, z] = m.vertices[*i];
            s += &format!("vertex {x} {y} {z}\n");
        }
        s.push_str("endloop\nendfacet\n");
    }
    s.push_str("endsolid test\n");
    s.into_bytes()
}
#[test]
fn binary_ascii_roundtrip_units_and_undo() {
    for bytes in [cube().stl().unwrap(), ascii(&cube())] {
        let node = stl::import(&bytes, "Cube", 1.).unwrap();
        assert_eq!(node.size().unwrap(), [20.; 3]);
        assert_eq!(geometry::bounds(&node).min.z, 0.);
        let p = project(node);
        assert_eq!(p, Project::parse(&p.bytes().unwrap()).unwrap());
        assert!((evaluate(&p).unwrap().volume() - 8000.).abs() < 1e-6);
        let output = evaluate(&p).unwrap().stl().unwrap();
        let n = stl::import(&output, "Again", 1.).unwrap();
        assert!((evaluate(&project(n)).unwrap().volume() - 8000.).abs() < 1e-6);
        let mut h = History::new(Project::default(), true);
        h.commit(p.clone()).unwrap();
        assert!(h.undo());
        assert!(h.project.parts.is_empty());
        assert!(h.redo());
        assert_eq!(h.project, p);
    }
    let n = stl::import(&cube().stl().unwrap(), "Inches", 25.4).unwrap();
    assert_eq!(n.size().unwrap(), [508.; 3]);
}
#[test]
fn binary_header_may_start_with_solid() {
    let mut bytes = cube().stl().unwrap();
    bytes[..5].copy_from_slice(b"solid");
    assert!(stl::import(&bytes, "cube", 1.).is_ok());
}
#[test]
fn reverse_winding_is_corrected() {
    let mut m = cube();
    for t in &mut m.triangles {
        t.swap(1, 2);
    }
    let n = stl::import(&m.stl().unwrap(), "reverse", 1.).unwrap();
    assert!((evaluate(&project(n)).unwrap().volume() - 8000.).abs() < 1e-6);
}
#[test]
fn rejects_open_duplicate_inconsistent_and_flat_meshes() {
    let mut open = cube();
    open.triangles.pop();
    let mut duplicate = cube();
    duplicate.triangles.push(duplicate.triangles[0]);
    let mut inconsistent = cube();
    inconsistent.triangles[0].swap(1, 2);
    for m in [open, duplicate, inconsistent] {
        assert!(stl::import(&ascii(&m), "bad", 1.).is_err());
    }
    let mut flat = cube();
    for v in &mut flat.vertices {
        v[2] = 0.;
    }
    assert!(stl::import(&ascii(&flat), "flat", 1.).is_err());
}
#[test]
fn rejects_malformed_nonfinite_and_limits() {
    let mut truncated = cube().stl().unwrap();
    truncated.pop();
    for b in [
        vec![],
        b"solid empty\nendsolid empty".to_vec(),
        truncated,
        vec![0; stl::MAX_STL_BYTES + 1],
        ascii(&cube()).replace_bytes_for_test(),
    ] {
        assert!(stl::import(&b, "bad", 1.).is_err());
    }
    for scale in [0., -1., f64::NAN, 100., 0.00001] {
        assert!(stl::import(&cube().stl().unwrap(), "bad", scale).is_err());
    }
    let mut binary = cube().stl().unwrap();
    binary[96..100].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(stl::import(&binary, "nan", 1.).is_err());
}
trait BadAscii {
    fn replace_bytes_for_test(self) -> Vec<u8>;
}
impl BadAscii for Vec<u8> {
    fn replace_bytes_for_test(self) -> Vec<u8> {
        String::from_utf8(self)
            .unwrap()
            .replace("endloop", "wrong")
            .into_bytes()
    }
}
#[test]
fn imported_mesh_cuts_groups_mirrors_and_resizes() {
    let n = stl::import(&cube().stl().unwrap(), "cube", 1.).unwrap();
    let mut p = project(n);
    let mut hole = Node::primitive("Cylinder");
    hole.hole = true;
    hole.position = p.parts[0].position;
    hole.shape = Shape::Cylinder {
        size: [5., 5., 30.],
    };
    p.parts.push(hole);
    let v = evaluate(&p).unwrap().volume();
    assert!(v < 8000. && v > 7000.);
    let ids = p.parts.iter().map(|n| n.id.clone()).collect();
    let group = operations::group(&mut p, &ids).unwrap();
    p.parts[0].rotation = [20., 30., 40.];
    let before = evaluate(&p).unwrap().volume();
    operations::mirror(&mut p, &Selection::from([group.clone()]), 0).unwrap();
    assert!((evaluate(&p).unwrap().volume() - before).abs() < 1e-5);
    operations::ungroup(&mut p, &group).unwrap();
    assert!((evaluate(&p).unwrap().volume() - before).abs() < 1e-5);
    p.parts[0].set_dimension(0, 40.);
    p.validate().unwrap();
    assert!(evaluate(&p).unwrap().volume() > before);
}
#[test]
fn loaded_mesh_indices_and_legacy_schema_are_checked() {
    let p = project(stl::import(&cube().stl().unwrap(), "cube", 1.).unwrap());
    let mut value = serde_json::to_value(&p).unwrap();
    value["parts"][0]["triangles"][0][0] = 99999.into();
    assert!(Project::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = serde_json::to_value(&p).unwrap();
    value["version"] = 4.into();
    assert!(Project::parse(&serde_json::to_vec(&value).unwrap()).is_err());
}
#[test]
fn self_intersecting_shells_rejected() {
    let mut m = cube();
    let mut other = cube();
    for v in &mut other.vertices {
        v[0] += 5.;
    }
    let offset = m.vertices.len();
    m.vertices.extend(other.vertices);
    m.triangles
        .extend(other.triangles.into_iter().map(|t| t.map(|i| i + offset)));
    assert!(stl::import(&m.stl().unwrap(), "intersecting", 1.).is_err());
}
