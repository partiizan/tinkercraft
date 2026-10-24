# TinkerCraft 0.16.0 — local modeling MVP

Rust-native portable packages target Apple Silicon macOS, Windows x64 and Linux x64.
Download from https://github.com/partiizan/tinkercraft/releases/latest and follow
QUICKSTART.md. See RELEASE-NOTES.md and checkpoints 16–17 for this release.

Publication requires all three native jobs to pass tests, static analysis, packaging,
extracted CLI execution and a real native-window framebuffer check. Linux additionally
runs mouse/keyboard modeling and recovery regression. These checks do not substitute
for hands-on Mac/Windows file-dialog and shortcut testing on user hardware.

Official Bambu Studio 02.08.02.61 imports the STL and 3MF fixture with the expected
80 × 30 × 5 mm dimensions, manifold geometry and two separate parts in the 3MF.
Generic RGB materials are not mapped automatically to Bambu filaments; manual
assignment is required. Physical printing has not been tested.

Mac bundles are ad-hoc signed and verified, but not Developer ID signed/notarized.
Windows executables are unsigned. No installer or automatic updater is included.

## Verified publication

[Native run 40143316634](https://github.com/partiizan/tinkercraft/actions/runs/40143316634)
passed all three native jobs and the release job on source commit
`deed5f89384c757fd960c1221d323fffa1e118d9`.
[Version 0.16.0](https://github.com/partiizan/tinkercraft/releases/tag/v0.16.0)
contains Apple Silicon Mac, Windows x64 and Linux x64 packages and checksums.
The Windows framebuffer probe captured 1024 × 768 pixels with 76,244 model pixels
and 1,682 distinct colors using Mesa llvmpipe. Mac and Linux used their default
platform graphics setup. Engine tests, Clippy, packaging checks and 3MF checks
passed on every target; Linux also passed reference-lib3mf and input regression.
The manual acceptance limitations above still apply.

## Build and verify on your native platform

Install the prerequisites from README.md, then from the source folder:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --release --workspace --locked
python3 scripts/test_packaging.py
python3 scripts/package.py
python3 scripts/verify_all_releases.py
```

On Windows use `python` if appropriate. Python 3.12 is used in CI. The package script
runs the native CLI self-test and desktop version probe, verifies binary platform
and architecture, writes a per-file SHA-256 manifest and an archive checksum.
The verifier detects missing/modified/extra files, validates bundle metadata and
executes the extracted package from a temporary folder outside the source tree.
These probes do not require an internet connection after the build.

Synthetic package-layout tests never count as native platform execution. The CLI
self-test checks legacy project migration, serialization and a Boolean/STL import
round trip through the actual Rust geometry engine. `--version` checks the desktop
entry point without opening a window; it is not a graphics test.

## Native desktop acceptance checklist

- Launch the actual packaged application; verify window, graphics and scaling.
- Create primitives and an imported STL; choose units and reject an invalid mesh.
- Rename/save the project; close/reopen it using real OS file dialogs.
- Move/rotate/resize; nudge on all axes, use snapping, drop a group, fit selection.
- Group/ungroup, mirror, cut holes, and undo/redo each major operation.
- Hide/show and lock/unlock; verify a lock blocks Delete, nudge and handles.
- Export an STL after editing a dimension; check its actual dimensions in a slicer.
  Hidden objects must still be in exported geometry, as the interface states.
- Restart and verify recovery, saved preferences and no overwrite of changed files.
- Check Cmd shortcuts on Mac / Ctrl shortcuts elsewhere; inspect native pickers.
- Run from the extracted package without the source tree and without network access.

Linux file-picker interaction was not exercised in the headless Xvfb container;
a working desktop portal backend is required. Bambu Studio model import is verified, but slicing and an actual print have not
been verified. No distribution certificate, installer or automatic updater is included.

## Checkpoint 11 hardening

Group drop-to-workplane uses positive source bounds before cuts, consistent with
alignment. Arrow keys nudge world X/Y; Page Up/Down nudge Z; Shift multiplies the
step by ten. Snap Off uses 1 mm for nudging. F fits the selected visible objects.
Cmd/Ctrl-D duplicates; Delete/Backspace deletes while no text field has focus.
Locked or hidden selections reject modeling operations.

Export commits the active draft and waits for the corresponding geometry result
before showing the save dialog, preventing export of a previous dimension value.
Pending export is cancelled on document replacement. Hide/lock state lives in
schema-7 projects. Preference changes do not mark project geometry as modified.

## Checkpoints 14–15

Appearance preferences (Light/Dark/System) include the viewport/grid. System
falls back to Dark if the OS theme is unavailable. Object colors are stored in
schema-8 projects; older schemas migrate with default colors. Old app versions
cannot open newly saved schema-8 projects.

3MF export preserves millimeter geometry as disjoint colored parts in one assembly.
Colors survive groups, cuts, transforms, save/reopen and undo. Recoloring a group
recolors its components; ungroup for separate colors. Later solids own overlapping
volume. Hidden objects remain included in export. STL remains geometry-only.

Independent validation uses ZIP/XML inspection and the 3MF Consortium lib3mf 2.5.0
strict reader, including oriented/manifold mesh checks. The native Linux regression
covers theme selection/persistence and color undo/redo/recovery. Bambu Studio
interactive import and final filament mapping still require hands-on acceptance.

Windows hosted-runner graphics use the checksum-pinned Mesa 26.2.4 software
OpenGL driver for the extracted-window test, because the Hyper-V display adapter
only exposes legacy OpenGL. These test-only DLLs are not shipped in the archive.
The check verifies the native Windows application/window/rendering path, not
compatibility with every physical GPU. Startup failures now display a useful
error dialog instead of silently exiting.
