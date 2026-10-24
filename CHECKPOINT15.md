# Checkpoint 15 — object colors and 3MF export (0.15.0)

Per-object RGB colors, recursive group recoloring, saved schema-8 projects and
undo/redo. Legacy projects default to blue-grey. Colored groups remain editable.
Grouped components retain their colors until explicitly recolored as a group.

Colored Boolean partitions retain cut geometry and avoid overlapping print volumes;
later solids own overlaps. Every color exports as closed mesh parts in one 3MF
assembly with base-material display colors. Units are millimeters. Export snapshots
the project and runs on a worker. Existing STL export is retained.

Validation includes colored volume agreement with ordinary Booleans, zero overlap,
transformed nested groups, schema migration, undo/reload, archive structure, and
independent XML checks. CI uses the 3MF Consortium lib3mf 2.5.0 strict reader on Linux.
Native build evidence is recorded below.

Bambu Studio interactive import, final filament mapping and a multicolor print
remain manual acceptance tasks. No claim is made that display colors configure AMS
slots. 3MF import, textures and per-face painting are outside this checkpoint.

Reference: https://github.com/3MFConsortium/spec_core

## Verified build

[Native validation run](https://github.com/partiizan/tinkercraft/actions/runs/40141986776), source
`a0a886789aed6e0911f08ccfd55d6d3040f440ba`. macOS ARM64, Windows x64 and Linux x64
all passed. All 46 core tests, Clippy, formatting, packaging tests and extracted
package probes passed. Mac ad-hoc signatures were verified before and after
packaging. Linux native UI checks include theme persistence, color undo/redo and
recovery. lib3mf 2.5.0 strict reading produced no warnings and confirmed the
fixture meshes are manifold and oriented. The two-color plate volume is
11803.790612309556 mm³. Bambu Studio manual filament mapping remains unverified.

Download artifacts from the run above: TinkerCraft-macOS-ARM64,
TinkerCraft-Windows-X64, or TinkerCraft-Linux-X64. Extract the outer workflow ZIP
and the native archive inside. This is version 0.15.0.
