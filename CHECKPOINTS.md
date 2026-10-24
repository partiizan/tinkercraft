# TinkerCraft checkpoints 10–12 — 0.11.0

## Current release-candidate policy

New main-branch builds publish as GitHub **Pre-releases**, versioned `0.17.0-rc.N`,
after all three native jobs pass. Manual workflow runs on main do the same;
pull requests and failed builds do not publish. A rerun keeps the same candidate
number so partial retries cannot mix different platform versions. Already published
assets are never replaced. The existing v0.16.0 release remains historical.

[Issue #3](https://github.com/partiizan/tinkercraft/issues/3) tracks checkpoints
18 (desktop acceptance), 19 (Bambu/printing acceptance, including #2), and 20
(explicit stable-release approval). The items below describe prior implementation
milestones; they do not imply that those manual acceptance gates have passed.
There is no automatic stable-release promotion in the workflow.


## Checkpoint 10: complete

- Undoable, saved Hide/Show and Lock/Unlock controls, with Show all.
- Hiding is viewport-only: hidden solids and holes stay in cuts and STL export.
- Locks protect edits/deletion at the history boundary; a locked child protects its
  top-level group. Hidden/locked objects are excluded from Select all.
- Full-width selectable object rows.
- Saved snap, tool, projection and optional camera preferences, with safe handling
  of invalid settings files. Project format advances to schema 7, reading 1–6.

## Checkpoint 11: release preparation complete; platform acceptance remains open

- Group drop-to-workplane, world-axis keyboard nudging, fit-selection, duplicate
  and delete shortcuts. Locks apply to these operations.
- Export commits pending numeric edits and waits for updated geometry.
- Native CLI self-test and desktop version probe.
- Target-aware packaging, per-file SHA-256 inventories, archive checksums,
  Mac bundle metadata checks and extracted-package runtime verification.
- CI workflow for native builds/tests/packages on Linux, Mac and Windows.

The full platform-verification checkpoint cannot be called complete until the
Mac/Windows runtime and native file-picker checks in RELEASE.md have been run.
This delivery is a local-modeling MVP release candidate, not a verified release
for all three operating systems.

## Verification

- 37 engine regression tests pass, including all previous tests and 8 new
  visibility/locking/preferences/group-drop/nudge scenarios.
- Four packaging regression tests pass; layout tests cover Linux, Windows and Mac
  using synthetic executable headers. They are not native platform execution.
- Native Linux release build and UI automation pass, including imported meshes,
  resize undo/cancel, recovery restore, hide/show, locked shortcut rejection,
  keyboard nudging, and persisted camera/tool settings.
- Group drop undo/redo and fit-selection additionally exercised in the native UI.
- Rustfmt and Clippy with warnings denied pass.
- Mac ARM and Windows GNU/MSVC cross-target compile checks pass.
- The real Linux archive passes inventory/checksum verification and both extracted
  executables pass their runtime probes outside the source tree.

## What the download contains

TinkerCraft/ contains complete Rust source, tests, examples and build instructions.
prebuilt/ contains the tested Linux x86-64 release package and its checksum.
history/ contains a Git bundle preserving checkpoints 7–11. No prebuilt Mac or
Windows application is included. Follow README.md and RELEASE.md to build locally.

Only local modeling/object management for 3D printing is in scope. Circuits,
code blocks, classrooms, accounts and cloud collaboration remain excluded.
Pause after this delivery for the user's next weekly-usage reading.

## Checkpoint 12: native build and package validation complete

Source transferred to partiizan/tinkercraft. All three native GitHub Actions jobs
passed at commit 8278f72e999c79c50b488c57695569d62de3e877. Fixed the Linux
runner's missing libxkbcommon-x11 runtime dependency and documented it for users.
Mac ARM64, Windows x64 and Linux x64 packages are available as workflow artifacts.
See CHECKPOINT12.md for evidence and download instructions. Interactive Mac/Windows,
OS file-dialog and slicer acceptance remain open.

## Checkpoint 13: complete — 0.13.0

Illustrated primitive palette, automatic cut-result display after successful
Group/button or keyboard, matte toolbox app icon, and assembled Mac bundle signing.
All three native jobs passed, including staged/extracted Mac codesign validation
and Linux GUI regression. See CHECKPOINT13.md for downloads and remaining manual
acceptance. Ad-hoc signing does not replace Developer ID signing/notarization.

## Checkpoints 14–15 — 0.15.0

14: Light/Dark/System preferences, persistent across restarts, with themed viewport
and grid. System follows OS events where supported, falling back to Dark.

15: Saved object colors, grouped multicolor cut geometry, and 3MF export through
UI and CLI. Schema 8 reads legacy files. 3MF assembly/material resources use mm;
colored parts partition geometry without overlap. New example: multicolor-plate.

See CHECKPOINT14.md, CHECKPOINT15.md and RELEASE.md for validation and limitations.
Bambu Studio filament mapping and a multicolor print remain manual acceptance tasks.

## Checkpoints 16–17 — 0.16.0

16: Extracted native-window/framebuffer gates on all platforms, verified Bambu
Studio STL/3MF import dimensions and manifold geometry, and safe close during
background 3MF export.

17: Versioned release archives/checksums, gated GitHub Release publishing, quick
start and scoped MVP handoff. See CHECKPOINT16.md and CHECKPOINT17.md for evidence
and the remaining hands-on OS/filament/print acceptance limits.

## Checkpoints 18–19: automation advanced; manual acceptance open

18 adds native GTK portal Save/Open/Save As/import/export checks and file-workflow
shortcuts. 19 adds reproducible Bambu slicing with resolved X1C/PLA presets and
programmatic two-filament assignment. See CHECKPOINT18.md, CHECKPOINT19.md and
ACCEPTANCE.md. Physical Mac/Windows and print evidence remain required in #3.
