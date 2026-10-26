This candidate completes the eight planned remaining modeling checkpoints (31–38):
custom workplanes, SVG extrusion, font-outline lettering, viewport editing inside
nested groups, point snapping, richer selection/navigation/accessibility, GPU
rendering and release hardening.

F4 opens Modeling studio. F3 enters a selected group; Shift-F3 applies and exits.
F1 opens the keyboard reference. Shift-drag adds a box selection; Alt-Shift-drag
removes it. 1/2/3/4 select top/front/right/isometric views. Save and export apply
active group workspaces; recovery always retains the full project.

SVG/font conversion runs in the background and embeds generated geometry in the
saved project. Custom planes persist in schema 11; schemas 1–10 still load. GPU
rendering caches geometry across camera movement, with a selectable software
fallback. UI scale and renderer preference persist. Native accessibility support
is enabled. Exact float parsing preserves generated meshes across save/reopen.

Read PARITY-ACCEPTANCE.md before sign-off. These checkpoints complete the planned
scope; full parity is still unconfirmed. Imports and scene complexity have explicit
limits, including plain SVG geometry and Latin font lettering. Hands-on hardware,
assistive technology, slicer/filament mapping and physical printing remain open.
This is a release candidate, not an automatically promoted stable release.

This candidate delivers modeling checkpoints 26–30:

- Tube, torus and regular polygon-prism shapes, with editable bore ratio or sides.
- Editable block-text labels that can be solids or engraving holes, including inside
  groups. Supports A–Z, digits, spaces and - . _ +, up to 24 characters per label.
- Alignment to a stationary reference object, with independently chosen source and
  target bounds edges and an offset. Locked references can remain fixed.
- Move, rotate and uniformly scale selected objects together without grouping them.
- Searchable object browser, nested component inspection, bulk selection and rename.

Open Modeling tools (M) for reference alignment and selection transforms. More
shapes and Object browser are in the left panel. The bundled modeling-catalog
example demonstrates raised and recessed lettering and all three new shapes.

Projects now save as schema 10. Schemas 1–9 still load; keep older project copies
if you need to open them with an older TinkerCraft version. No new dependencies.
Block lettering is a built-in font style; arbitrary fonts and SVG import remain
future work. Polygon diameter is its vertex-circle diameter. Reference alignment
uses source bounding boxes, not exact surface contact. Selection scaling is uniform.

This is a release candidate. Hands-on OS, interactive slicer and print acceptance
remain deferred at the user's request; stable promotion still needs an explicit
release decision. See CHECKPOINT26-30.md and issue #3.

Previous candidate features and release policy follow.

This candidate adds five modeling checkpoints (21–25): measurements and a
workplane ruler, numeric editing of nested group components, equal-center/equal-gap
distribution, linear/radial repeated copies, and adjustable curve resolution.
Open Modeling tools with M; select a group and press F2 to edit its contents.
Group edits preserve the group transform and apply as one undo step.

Measurements use world-axis source bounds before cuts, not exact surface clearance.
Point picking uses the Z=0 workplane; numeric ruler points support arbitrary XYZ.
Radial copies rotate around world Z. Curve resolution defaults to the previous
96 segments and is stored in schema 9. Older projects load without changing their
default resolution; older app versions cannot reopen schema-9 saves.

At the user's request, hands-on OS/slicer/print acceptance is deferred until this
feature batch is complete. It remains an explicit gate before stable release.
See CHECKPOINT21-25.md for scope and validation details.

This candidate adds real Linux GTK file-dialog regression and keyboard shortcuts
for STL import (Cmd/Ctrl-I), STL export (Cmd/Ctrl-Shift-E) and 3MF export
(Cmd/Ctrl-Alt-E). Bambu CLI validation now includes actual single- and two-filament
slicing. See CHECKPOINT18.md, CHECKPOINT19.md and ACCEPTANCE.md for evidence and
remaining hands-on checks; neither checkpoint is fully signed off yet.

This is a **release candidate**, not a stable release. Each successful main-branch
build receives a unique `0.20.0-rc.N` version; the tag and downloads identify the
exact candidate. The number is the workflow run number, not a completed-checkpoint
count. Rerunning a workflow retains its candidate identity across all platforms.

Track remaining acceptance and report candidate-specific results in
https://github.com/partiizan/tinkercraft/issues/3.

Remaining gates: native desktop dialogs/shortcuts on user hardware, interactive
Bambu filament assignment and a physical print, followed by an explicit stable
release decision. Closing issues never automatically promotes a build to stable.

TinkerCraft 0.16.0 completes the implementation and automated validation checkpoints
for the local modeling MVP. Rust-native packages target Apple Silicon macOS,
Windows x64 and Linux x64. Circuits, code blocks and cloud features are out of scope.

- Editable Boolean groups: solids and holes cut automatically, and the resulting
  group supports further transforms, cuts, grouping and undo.
- Primitive modeling, STL import, numeric/handle transforms, snapping, alignment,
  mirroring, visibility/locks, saved projects and recovery.
- Light/Dark/System themes, object colors and generic color-part 3MF export.
- Quitting waits for an active 3MF export instead of interrupting its worker.
- Versioned downloads, per-file manifests and archive SHA-256 checksums.

Publishing is gated on all three native build/test/package jobs, including opening
an extracted packaged app and checking its rendered framebuffer. Linux also runs
mouse/keyboard modeling and recovery regressions. Official Bambu Studio 02.08.02.61
imports the STL and 3MF fixture with correct dimensions and manifold geometry.

Read QUICKSTART.md inside the archive. Mac bundles are ad-hoc signed and verified,
not Developer ID signed/notarized; Windows executables are unsigned. These are
portable packages, not installers. A modern OpenGL driver is required.

Hands-on Mac/Windows file-dialog/shortcut acceptance and physical printing remain
unverified. Generic RGB materials are not an automatic Bambu filament/AMS mapping:
Bambu's CLI preserves the parts but drops their RGB materials on re-export. Assign
filaments in the slicer. This release does not claim full Tinkercad modeling parity.

Windows hosted-runner graphics use the checksum-pinned Mesa 26.2.4 software
OpenGL driver for the extracted-window test, because the Hyper-V display adapter
only exposes legacy OpenGL. These test-only DLLs are not shipped in the archive.
The check verifies the native Windows application/window/rendering path, not
compatibility with every physical GPU. Startup failures now display a useful
error dialog instead of silently exiting.
