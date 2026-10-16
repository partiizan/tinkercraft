# TinkerCraft checkpoints 10–12 — 0.11.0

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
