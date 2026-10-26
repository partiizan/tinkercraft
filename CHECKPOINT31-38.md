# Checkpoints 31–38 · remaining planned modeling features

31 — Custom workplanes: numeric origin/rotation or picking a solid face; persistent
project state; grid, new shapes, drop-to-plane and ruler picking use that plane.
Reset to XY and look along the plane normal. Object numeric coordinates remain
world coordinates. A group-editing workspace uses its own temporary local plane.

32 — SVG extrusion: import filled paths/shapes and expanded strokes, SVG transforms,
nonzero/even-odd cutouts, and chosen width/depth. Conversion runs in a worker and
requires Add generated mesh. It embeds a closed editable mesh in the project.
The SVG is interpreted as monochrome geometry; apply an object color afterwards.
Raster images, SVG text, clips/masks/filters and external references are rejected;
convert those constructs to plain vector outlines in the source application.
Limits: 1 MB input, 2,000 XML elements, 32 XML levels, 12,000 flattened outline
points and the existing project mesh triangle budget. Curves use 16 subdivisions
per quadratic and 24 per cubic segment; compare curved logos at their final size.

33 — Font lettering: choose TTF/OTF, enter Latin text (up to 48 characters), choose
width/depth and generate an embedded mesh. Outline curves, glyph advances and
legacy horizontal kerning are supported. Complex-script shaping, ligature layout
and variable-font editing are outside this importer's contract; supply SVG paths
for those. The font is not embedded or needed on reopen. Regenerate to change the
text/font; the existing block-text primitive remains parametrically editable.
20 MB font input and 12,000 outline points. Very detailed text can exceed the
20,000-triangle project mesh budget and is rejected with an explanation.

34 — Viewport editing inside groups: F3 enters selected group contents in local
coordinates. Existing move/rotate/resize handles, numeric controls, holes, colors,
add/remove and undo operate on children. Nested groups can be entered again.
Shift-F3 applies and exits; Cancel group edits discards that workspace. Applying
one scope makes one parent undo step. Parent transforms and child IDs are retained.
Save, export, opening another project and normal close apply active scopes first.
Recovery always serializes the validated full project, never an isolated fragment.
The numeric F2 component editor remains available.

35 — Object snapping: select objects, pick a source point on the selection, then a
target point on another object. Choose exact triangle-surface intersection or the
nearest vertex of the hit triangle. The entire selection translates together in
one undo step. Escape cancels and stale scene revisions reject a pending snap.
This is mesh-based point snapping, not analytic CAD constraints or edge inference.

36 — Selection/navigation/accessibility: Shift-drag adds objects whose projected
source-bounds centers lie in the box; Alt-Shift-drag removes them. Hidden/locked
objects are excluded. 1/2/3/4 select top/front/right/isometric views. F1 shows a
keyboard reference, F4 opens the studio. Interface scale (0.8–1.6) persists.
Native AccessKit support is enabled; custom palette buttons have accessible names.
Numeric controls offer keyboard alternatives to viewport handles. Screen-reader
behavior still requires hands-on testing on each OS.

37 — GPU viewport: native OpenGL depth-tested triangles, lighting, hole hatching
and a workplane grid. GPU geometry buffers are cached across camera movement;
ray/triangle picking replaces per-pixel CPU ID rendering for normal interaction.
The software renderer remains selectable and the preference persists. GPU init
failure falls back to software. Packaged-window tests require successful GPU
initialization and actual colored model pixels in the captured framebuffer on
all three operating systems. Existing model/node limits remain in force.

38 — Release hardening and scope closure: schema 11 adds workplanes while reading
schemas 1–10. Exact JSON float parsing prevents imported mesh drift across saves.
Added workplane/picking/group/outline regression tests, native scoped-edit/recovery/
SVG/fallback tests, cross-platform native outline CLI fixture and strict 3MF reader
validation. Dependency licenses are refreshed. PARITY-ACCEPTANCE.md is the final
hands-on checklist; no stable or literal Tinkercad-wide parity claim is automatic.

## Local validation before publication

66 core tests, strict Clippy, formatting, release compilation and packaging tests
passed. Native GPU output was captured (126,772 model-colored pixels in the local
1440×960 smoke frame). Existing native UI regression passed. The new studio test
passed group handle editing, full-project recovery, apply/undo/redo/cancel, custom
plane placement, native SVG generation and software-fallback preference checks.
Native SVG/font CLI fixture exports two colored meshes and revalidates as 3MF.

## Scope boundary

These eight checkpoints finish the previously planned local-modeling feature batch.
The scoped inventory and known limits are in PARITY-ACCEPTANCE.md. Manual checks
may uncover defects or missing workflows; any such finding blocks sign-off and
requires another candidate. Passing this checklist does not prove equivalence to
all of Tinkercad's evolving shape catalog, generators or modeling behavior.
