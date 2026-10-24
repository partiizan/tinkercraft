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
Native build results will be recorded after validation.

Bambu Studio interactive import, final filament mapping and a multicolor print
remain manual acceptance tasks. No claim is made that display colors configure AMS
slots. 3MF import, textures and per-face painting are outside this checkpoint.

Reference: https://github.com/3MFConsortium/spec_core
