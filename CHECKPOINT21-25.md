# Modeling checkpoints 21–25 · 0.18 release candidates

21 — Measurement tools: selected source bounds, bounds-center distance and per-axis
bounding-box gaps for two objects, plus an independent two-point 3D ruler. Numeric
XYZ input or snapped Z=0 workplane picks. These are not exact surface-clearance
measurements; bounds include source geometry before Boolean subtraction.

22 — Edit group contents: F2 opens a staged numeric editor for nested components.
Edit names, local positions/rotations, primitive dimensions, holes, colors and
nested uniform scales. Apply commits once, preserves the root transform, recomputes
cuts, and supports undo/redo. Cancel discards the draft. Locked groups are protected.
This is numeric component editing; direct child viewport handles are future work.

23 — Distribution: three or more selected top-level objects, X/Y/Z axis, equal
centers or equal bounding-box gaps. End objects remain fixed; negative gaps describe
overlap. Hidden or locked selections are rejected without changing the document.

24 — Repeated copies: linear XYZ offset or radial sweep about world Z with optional
orientation rotation. Full circles avoid a duplicate endpoint; partial arcs include
the end angle. Copies preserve nested holes and colors and receive fresh IDs.
The existing 100-node project limit is checked before cloning.

25 — Curve resolution: 24/48/96/192 segment presets for cylinders/cones and spheres
(spheres use half as many longitudes and a quarter as many latitude divisions).
Applies to selected primitives or individual group children. Preview, cuts, bounds,
STL and 3MF use the chosen tessellation. Default 96 preserves legacy geometry.

## Validation and release gate

Core regression tests cover distribution anchors/gaps and undo, grouped-hole copies,
radial endpoints, component edits and stale drafts, schema migration, tessellation
and export, and workplane ray picking. Existing workspace tests and strict Clippy
remain required, together with all three native build/package jobs.

The five checkpoints are delivered together as one versioned release candidate.
Automated checks do not substitute for hands-on acceptance. Per user direction,
physical Mac/Windows testing, interactive slicer filament assignment and a real
print are deferred until the feature batch is delivered. Issue #3 tracks those
remaining gates and the explicit stable-release decision.

## Delivered candidate and evidence

Published **v0.18.0-rc.17**, source `ed977c92bf9f50dc2e3e1d3a8e136ec67449f0a0`:
https://github.com/partiizan/tinkercraft/releases/tag/v0.18.0-rc.17

All three native jobs and release publication passed in run 40221076627:
https://github.com/partiizan/tinkercraft/actions/runs/40221076627

Local validation: all 52 core regression tests, rustfmt, strict Clippy, release
compilation and four packaging tests passed. The Linux native regression passed,
then passed again with a new group-content scenario: select the demo plate and its
two holes, group, F2, enlarge one hole from 5 to 10 mm, Apply, Undo, Redo. Recovery
snapshots verified the child dimension at each step while retaining the group.
This scenario is now in scripts/native-smoke.py for subsequent builds. The RC17
CI run predates that test-script-only addition; application source is unchanged.

Artifacts target Apple Silicon macOS, Windows x64 and Linux x64. Hands-on hardware,
interactive slicer and physical-print acceptance remain open, as described above.
