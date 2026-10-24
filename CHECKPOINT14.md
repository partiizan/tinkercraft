# Checkpoint 14 — appearance preferences (0.14.0)

Preferences now offers Light, Dark and System. System is the default for new and
legacy preferences. The egui/winit native backend reports OS appearance changes;
when the platform does not report a theme, System falls back to Dark.

UI controls, viewport background/grid and viewport labels follow the resolved
theme. Changes are immediate, saved separately from projects, and do not change
geometry or undo history. Reset defaults restores System.

Core coverage: legacy preference migration, all three modes surviving restart,
and viewport theme changes retaining the same picking behavior. Native platform
and UI validation is recorded with checkpoint 15's combined build.

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
