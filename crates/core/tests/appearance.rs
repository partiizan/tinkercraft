use tinkercraft_core::{
    preferences::{Appearance, Preferences},
    viewport::{self, Camera},
};
#[test]
fn legacy_preferences_default_to_system_and_modes_roundtrip() {
    let mut old = serde_json::to_value(Preferences::default()).unwrap();
    old.as_object_mut().unwrap().remove("appearance");
    assert_eq!(
        serde_json::from_value::<Preferences>(old)
            .unwrap()
            .appearance,
        Appearance::System
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("preferences.json");
    for mode in [Appearance::Light, Appearance::Dark, Appearance::System] {
        let p = Preferences {
            appearance: mode,
            ..Preferences::default()
        };
        p.save(&path).unwrap();
        assert_eq!(Preferences::load(&path).unwrap(), p);
    }
}
#[test]
fn viewport_theme_changes_background_without_changing_picking() {
    let camera = Camera::default();
    let selected = Default::default();
    let light = viewport::render_themed(&camera, 64, 64, &[], &selected, None, false);
    let dark = viewport::render_themed(&camera, 64, 64, &[], &selected, None, true);
    assert!(light.rgba[0] > dark.rgba[0] + 100);
    assert_eq!(light.ids, dark.ids);
}
