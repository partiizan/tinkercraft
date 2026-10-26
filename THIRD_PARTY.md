# Third-party software

TinkerCraft is an independent project. It is not affiliated with ArtCraft,
Autodesk or Tinkercad. No ArtCraft application code or artwork is copied.
The engine-first/native Rust design follows the general architecture discussed
in ArtCraft's public repositories.

Primary dependencies (exact resolutions in Cargo.lock):

- egui / eframe 0.33.3 — MIT OR Apache-2.0; native UI and presentation.
- manifold-rust 0.16.0 — Apache-2.0; Rust port of the Manifold geometry kernel.
  https://github.com/larsbrubaker/manifold-rust
- glam 0.30 — MIT OR Apache-2.0; transforms and camera math.
- serde / serde_json — MIT OR Apache-2.0; project serialization.
- rfd 0.15 — MIT; native file dialogs (portal dialogs on Linux).
- uuid — Apache-2.0 OR MIT; unique document IDs.
- tempfile — MIT OR Apache-2.0; atomic writes.
- directories — MIT OR Apache-2.0; OS-specific app-data paths.
- zip 2.4.2 — MIT; 3MF ZIP packaging (stored entries, no compression codecs).
- anyhow — MIT OR Apache-2.0; error propagation.

Transitive dependencies retain their upstream licenses. Cargo downloads their
sources and license files into its registry cache. Review dependency-licenses.csv
and bundled dependency-licenses/ when distributing compiled artifacts. The
application uses eframe's bundled default fonts, under their upstream licenses.

Additional outline/accessibility capabilities in 0.20:

- usvg 0.45.1 — Apache-2.0 OR MIT; SVG parsing and path/stroke conversion.
- ttf-parser 0.25.1 — MIT OR Apache-2.0; local font outlines.
- roxmltree 0.20.0 — MIT OR Apache-2.0; bounded XML preflight.
- clipper2-rust 1.2.0 — BSL-1.0; winding-rule outline unions.
- AccessKit and OS adapters — MIT/Apache-2.0 as specified per crate in the inventory.

The tests use epaint's existing bundled fonts. User-chosen fonts are converted to
geometry, not copied into projects. Dependency license texts are included; AccessKit
and atspi-proxies workspace licenses were obtained from their upstream repositories
where the published crate did not include its workspace license file.
