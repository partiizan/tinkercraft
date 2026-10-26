# Checkpoints 26–30 · 0.19 release candidates

26 — Expanded shape catalog: Tube, Torus and regular Polygon prism. Tube/torus
bores are ratios of outer diameter (0.05–0.95); polygons have 3–32 sides. Circular
XY dimensions stay coupled. Polygon diameter means the circle through its vertices;
its measured width depends on side count and orientation. Torus Z sets tube height.
Curve resolution applies to tube/torus; torus ring divisions round up to a multiple
of four so all presets include the top/bottom extrema. Each shape supports transforms,
colors, solid/hole grouping, undo, STL and 3MF export.

27 — Editable block lettering: built-in 5×7 alphabet, A–Z (lowercase renders as
uppercase), digits, spaces and - . _ +. Labels support 1–24 characters, with a
256-character project total. Width/height fit the actual full label; depth is its
extrusion. Slight overlap connects diagonal strokes. Label text persists as a
parameter, including inside groups; mirroring preserves editable lettering. Use a
solid for raised letters or a hole for recessed lettering. Separate characters
need to intersect a base to form a connected printable object. Arbitrary fonts,
Unicode typesetting and SVG extrusion are not part of this checkpoint.

28 — Reference-object alignment: choose a visible top-level reference, world axis,
source edge (Min/Center/Max), target edge and signed offset. The reference never
moves, even when selected. It may remain locked; moving objects must be visible
and unlocked. Bounds describe source geometry before cuts, not surface clearance.

29 — Multi-object transforms: move, rotate and uniformly scale a selection while
keeping its objects independent. Default pivot is combined source-bounds center;
custom world XYZ pivot is optional. Apply scales and rotates around the pivot,
then adds translation. Nested group transforms and holes are preserved. One Apply
is one undo step. Nonuniform group scaling and multi-object viewport handles remain
future work.

30 — Object browser: case-insensitive search over names, shape types and nested
component names; expandable component trees; select matches, invert or clear;
Shift-click selection; rename a single object or number a batch in document order.
Search selects top-level roots. Hidden/locked objects are shown but excluded from
bulk selection. Visibility remains viewport-only, not export filtering.

## Access and validation

More shapes and Object browser are in the left panel. M opens Modeling tools,
with collapsible controls for the new shapes, selection transforms and reference
alignment. F2 remains the staged group-component editor. Parameter edits participate
in project validation, recovery, save and undo. Schema 10 reads schemas 1–9 and
stores the new shapes; older app versions cannot open schema-10 saves.

The new regression suite checks hollow volumes, polygon mirroring, printable letter
meshes and STL re-import, text cuts and group edits, locked reference alignment,
selection transforms and undo, atomic failure, nested search, renaming and schema
roundtrips. examples/modeling-catalog.tinkercraft demonstrates all new geometry.
The CI gates include exporting and validating its colored 3MF alongside the existing
fixture, on all three platforms, with the reference reader on Linux.

Hands-on platform acceptance, interactive slicer filament assignment and physical
printing are deferred per user instruction. Checkpoints 18–20 remain acceptance
and stable-release gates; implementation completion does not close them.

Local validation before publication: 60 core regression tests, rustfmt, strict
Clippy, release compilation and four packaging tests passed. Both Linux native UI
regressions passed. The new script edits a label, undoes/redoes it, searches and
renames through the browser, then moves four independent objects together and
undoes/redoes that transform. It is a required Linux CI gate for this candidate.
