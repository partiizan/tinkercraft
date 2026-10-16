# TinkerCraft 0.11.0 — local modeling MVP release candidate

This checkpoint completes release preparation, not certification on every OS.
The application targets native macOS (Apple Silicon), Windows x64 and Linux x64.
Circuits, code blocks and cloud/classroom functionality are excluded.

## Evidence and remaining gates

| Target | Compile/build | Runtime | Package |
|---|---|---|---|
| Linux x86-64 | Native CI release build passed | Engine tests, packaged CLI and native X11 GUI regression passed | Real archive verified and extracted executables tested |
| macOS Apple Silicon | Native CI release build passed | Engine tests and packaged CLI passed; interactive GUI pending | Real ARM64 app bundle verified |
| Windows x64 | Native MSVC CI release build passed | Engine tests and packaged CLI passed; interactive GUI pending | Real x64 ZIP verified |

Checkpoint 12 evidence: [successful native run](https://github.com/partiizan/tinkercraft/actions/runs/38957322621),
source commit `8278f72e999c79c50b488c57695569d62de3e877` (2026-10-16).
All three jobs passed formatting, 37 engine tests, Clippy, four packaging tests,
release compilation, packaging and extracted executable probes. Linux also passed
its automated native window regression. These checks do not attest Mac or Windows
GUI behavior or real file-dialog/slicer integration.

Download native packages from that run's Artifacts section; see CHECKPOINT12.md.
Before calling this a cross-platform MVP release, finish the desktop acceptance
checklist below. Packages are unsigned and unnotarized.
The documentation bundled in the initial CI archives predates these results;
this repository's RELEASE.md and CHECKPOINT12.md contain the updated evidence.

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
a working desktop portal backend is required. Bambu Studio slicing and an actual
print have not been verified. No code-signing, installer or automatic updater is
included in this local-tool MVP candidate.

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
