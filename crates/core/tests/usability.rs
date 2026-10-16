use tinkercraft_core::{
    geometry,
    operations::{self, Selection},
    preferences::Preferences,
    *,
};
#[test]
fn hide_is_view_only_and_persists() {
    let mut p = Project::demo();
    let volume = evaluate(&p).unwrap().volume();
    p.parts[0].hidden = true;
    assert_eq!(geometry::sources(&p).len(), 2);
    assert_eq!(evaluate(&p).unwrap().volume(), volume);
    assert_eq!(p, Project::parse(&p.bytes().unwrap()).unwrap());
    for n in &mut p.parts {
        n.hidden = true;
    }
    assert!(geometry::sources(&p).is_empty());
    assert!(evaluate(&p).unwrap().stl().is_ok());
}
#[test]
fn hidden_group_inherits_visibility_without_removing_cuts() {
    let mut p = Project::demo();
    let ids = p.parts.iter().map(|n| n.id.clone()).collect();
    operations::group(&mut p, &ids).unwrap();
    let volume = evaluate(&p).unwrap().volume();
    p.parts[0].hidden = true;
    assert!(geometry::sources(&p).is_empty());
    assert_eq!(evaluate(&p).unwrap().volume(), volume);
    p.parts[0].hidden = false;
    if let Shape::Group { children, .. } = &mut p.parts[0].shape {
        children[0].hidden = true;
    }
    assert_eq!(geometry::sources(&p).len(), 2);
}
#[test]
fn locks_block_modeling_removal_and_combined_unlock_edit() {
    let mut p = Project::demo();
    p.parts[0].locked = true;
    let mut h = History::new(p.clone(), true);
    for operation in 0..6 {
        let mut next = p.clone();
        match operation {
            0 => {
                next.parts.remove(0);
            }
            1 => next.parts[0].position[0] += 1.,
            2 => next.parts[0].name = "changed".into(),
            3 => next.parts[0].hole = true,
            4 => next.parts[0].set_dimension(0, 90.),
            _ => {
                next.parts[0].locked = false;
                next.parts[0].position[0] += 1.;
            }
        }
        assert!(h.commit(next).is_err());
        assert_eq!(h.project, p);
    }
    let mut next = p.clone();
    next.parts[0].hidden = true;
    h.commit(next.clone()).unwrap();
    next.parts[0].locked = false;
    h.commit(next.clone()).unwrap();
    next.parts[0].position[0] += 1.;
    h.commit(next).unwrap();
    h.undo();
    h.undo();
    assert!(h.project.parts[0].locked);
    h.redo();
    assert!(!h.project.parts[0].locked);
}
#[test]
fn locked_children_protect_parent_and_can_be_unlocked_recursively() {
    let mut p = Project::demo();
    let ids = p.parts.iter().map(|n| n.id.clone()).collect();
    let id = operations::group(&mut p, &ids).unwrap();
    if let Shape::Group { children, .. } = &mut p.parts[0].shape {
        children[0].locked = true;
    }
    let mut h = History::new(p.clone(), true);
    let mut next = p.clone();
    next.parts[0].position[0] += 1.;
    assert!(h.commit(next).is_err());
    let mut next = p.clone();
    operations::ungroup(&mut next, &id).unwrap();
    assert!(h.commit(next).is_err());
    let mut next = p.clone();
    next.parts[0].set_locked_recursive(false);
    h.commit(next).unwrap();
    let mut next = h.project.clone();
    operations::ungroup(&mut next, &id).unwrap();
    h.commit(next).unwrap();
    assert_eq!(h.project.parts.len(), 3);
}
#[test]
fn cannot_group_or_mirror_locked_parts() {
    let mut p = Project::demo();
    p.parts[0].locked = true;
    let h = History::new(p.clone(), true);
    let mut next = p.clone();
    let ids = next.parts.iter().map(|n| n.id.clone()).collect();
    operations::group(&mut next, &ids).unwrap();
    assert!(h.clone().commit(next).is_err());
    let mut next = p;
    next.parts[0].rotation = [20., 30., 40.];
    let h = History::new(next.clone(), true);
    let ids = Selection::from([next.parts[0].id.clone()]);
    operations::mirror(&mut next, &ids, 0).unwrap();
    assert!(h.clone().commit(next).is_err());
}
#[test]
fn preferences_roundtrip_and_invalid_file_preservation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("preferences.json");
    let p = Preferences {
        move_snap: 0.5,
        angle_snap: 45.,
        tool: 2,
        orthographic: true,
        yaw: 1.2,
        ..Preferences::default()
    };
    p.save(&path).unwrap();
    assert_eq!(p, Preferences::load(&path).unwrap());
    for invalid in [
        "{broken",
        "{\"version\":2}",
        "{\"tool\":255}",
        "{\"distance\":0}",
        "{\"move_snap\":2}",
    ] {
        std::fs::write(&path, invalid).unwrap();
        assert!(Preferences::load(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), invalid);
    }
    std::fs::write(&path, "{}").unwrap();
    assert_eq!(Preferences::load(&path).unwrap(), Preferences::default());
}

#[test]
fn group_drop_and_nudge_preserve_relative_geometry() {
    let mut p = Project::demo();
    let ids = p.parts.iter().map(|n| n.id.clone()).collect();
    let id = operations::group(&mut p, &ids).unwrap();
    p.parts[0].rotation = [25., 35., 45.];
    p.parts[0].position[2] = 50.;
    let ids = Selection::from([id]);
    let v = evaluate(&p).unwrap().volume();
    operations::drop_to_plane(&mut p, &ids).unwrap();
    assert!(geometry::bounds(&p.parts[0]).min.z.abs() < 1e-8);
    let original = p.clone();
    operations::translate(&mut p, &ids, [1., -2., 3.]).unwrap();
    assert!((evaluate(&p).unwrap().volume() - v).abs() < 1e-5);
    let mut h = History::new(original.clone(), true);
    h.commit(p).unwrap();
    h.undo();
    assert_eq!(h.project, original);
}
#[test]
fn nudge_and_drop_of_locked_group_are_rejected_at_commit() {
    let mut p = Project::demo();
    let ids = p.parts.iter().map(|n| n.id.clone()).collect();
    let id = operations::group(&mut p, &ids).unwrap();
    p.parts[0].position[2] += 10.;
    p.parts[0].locked = true;
    let ids = Selection::from([id]);
    let h = History::new(p.clone(), true);
    let mut next = p.clone();
    operations::drop_to_plane(&mut next, &ids).unwrap();
    assert!(h.clone().commit(next).is_err());
    let mut next = p;
    operations::translate(&mut next, &ids, [1., 0., 0.]).unwrap();
    assert!(h.clone().commit(next).is_err());
}
