# TinkerCraft — native Rust edition 0.11

A local, cross-platform solid-modeling editor. The application and modeling engine
are written in Rust. It uses egui/eframe for the native desktop interface and the
pure-Rust `manifold-rust` geometry kernel. There is no browser, JavaScript runtime,
Three.js, Tauri, hosted service, account, or telemetry in this edition.

This is a local modeling MVP in development, focused on simple objects for 3D
printing. Circuits, code blocks, cloud services and classroom tools are out of scope.
Checkpoints 10–11 add visibility/locks, saved preferences and release preparation.
This is an MVP release candidate; native Mac/Windows acceptance checks remain open.

## Run

Install Rust using https://rustup.rs, then run these commands inside this folder:

```sh
cargo test --workspace --locked
cargo run --release -p tinkercraft --locked
```

The pinned toolchain is installed automatically by rustup. Dependencies are locked
in Cargo.lock. The first build requires internet access; running the application
and editing/exporting models do not.

- **macOS:** install Xcode Command Line Tools with `xcode-select --install`.
  Apple Silicon is supported. Run from an ARM-native terminal. Native Intel builds
  are possible with an Intel Rust toolchain, but no Intel Mac build is verified here.
- **Windows:** use the default MSVC Rust toolchain and install Visual Studio Build
  Tools with Desktop Development with C++ and a Windows SDK.
- **Linux:** an X11 display or XWayland, OpenGL-capable driver, and XDG desktop portal
  with a working file-picker backend are required. On Debian/Ubuntu install
  `build-essential pkg-config libx11-dev libxi-dev libxrandr-dev libxcursor-dev
  libxinerama-dev libgl1-mesa-dev libxkbcommon-dev libxkbcommon-x11-0 libwayland-dev xdg-desktop-portal`.
  Your desktop should supply the matching portal backend (GTK, KDE, etc.).

## Build distributable packages

```sh
cargo build --release --workspace --locked
python3 scripts/package.py
python3 scripts/verify_all_releases.py
```

On Windows, use `python` if `python3` is unavailable. The script packages the build
for the operating system on which it runs:

| Host | Package |
|---|---|
| macOS | ZIP containing `TinkerCraft.app` and the CLI |
| Windows | ZIP containing `tinkercraft.exe` and `tinkercraft-cli.exe` |
| Linux | tar.gz containing both native executables |

`.github/workflows/build.yml` runs tests and native release builds on macOS,
Windows and Linux, then uploads packages as workflow artifacts.
See [GitHub Actions](https://github.com/partiizan/tinkercraft/actions) and
CHECKPOINT12.md for native build results and download instructions.
The packages do not contain signed installers.
Mac signing/notarization and Windows Authenticode signing are not configured.
Do not distribute development packages as signed or production-verified releases.

## Modeling

- Boxes, cylinders, spheres, cones, wedges and roofs, with numeric dimensions and center positions.
- Source solids/holes and real nested union/subtraction. Show cut result displays
  the printable result. Source recipes stay editable.
- Select rows or click shapes. Shift-click toggles selection. Select all or Cmd/Ctrl-A.
- Drag background to orbit, right-drag to pan, scroll to zoom. Iso/Top/Front/Right
  change direction. Orthographic toggles projection; Fit all resets framing.
- W selects Move, E selects Rotate, R selects Resize. Drag a colored axis handle. In Rotate mode,
  dragging the axis handle changes rotation about that world axis; these are axis
  handles rather than the previous version's rotation rings. Release commits one
  undo step. Escape or focus loss cancels. Move and angle snapping are configurable.
- Resize uses square handles along the object's local axes, anchored at its center
  (group pivot). Drag outward to increase size; the dimension changes twice the
  axis displacement. Linear snap applies to the resulting dimension. Cylinders and
  cones keep equal X/Y diameters, spheres resize uniformly, and groups scale uniformly.
  Rotate the view to access an axis pointing toward the camera. Resizing can move a
  base below Z0; use Drop to workplane for individual objects after resizing.
- Dimensions and positions are millimeters; angles are degrees. Rotation applies
  world X, then Y, then Z, matching previous files.
- Group a selection containing at least two objects and one solid. Groups support
  movement, rotation and uniform numeric or handle scale. Ungroup to edit components.
- Duplicate/delete selections; drop primitives, meshes and groups to the XY workplane.
- Arrow keys nudge world X/Y; Page Up/Down nudge Z. Shift multiplies the snap step
  by ten (Snap Off uses 1 mm). F fits the selection. Cmd/Ctrl-D duplicates;
  Delete/Backspace deletes when no text field has focus.
- Align min/center/max on X/Y/Z to the selected set's combined source bounds.
- Mirror selected objects together about their source-bounds center. This edits in
  place: duplicate first to retain the original. Cone apex reversal is supported.
- Cmd/Ctrl-G groups; Shift-Cmd/Ctrl-G ungroups. Cmd/Ctrl-Z undoes;
  Shift-Cmd/Ctrl-Z redoes. Undo history is limited to 100 steps per session.

Ungrouping bakes parent transforms into children but returns holes to sibling
scope, where they can cut other overlapping solids. Make hole groups Solid before
ungrouping. Alignment is based on positive source geometry before cuts. Group drop uses positive source bounds before cuts. There is no nonuniform group scale.

## Visibility, locks and preferences

Select an object row and use Hide/Show or Lock/Unlock. Show all restores hidden
objects. Hide affects only the edit viewport; hidden solids and holes still
participate in the result and exported STL. There is no export-exclusion toggle.
Lock blocks shape edits, transforms, deletion and grouping. A locked child protects
its entire group; Unlock recursively releases it. Visibility changes and undo/redo
remain available. Select all omits hidden/locked objects; select their full-width
list rows explicitly to show/unlock them. Flags are saved with the project.

Preferences saves tool, snap, projection and (optionally) the camera between
sessions. These are local user settings, separate from geometry and undo history.
Invalid preferences are retained; Reset defaults explicitly replaces them. The
config directory is selected through the OS-standard directories API.

## Files and recovery

Open `.tinkercraft` or legacy `.shapeworkshop` files from checkpoints 1–6. Versions
1–6 migrate in memory to version 7; imported mesh geometry is embedded; saved files keep the established internal
`shapeworkshop` identifier. Empty and hole-only documents can be saved even when
no printable result exists. Limits remain 100 nodes total, 8 nested group levels,
0.01–1000 mm dimensions, ±10000 mm centers, and a 32 MB project file, and 20,000 imported triangles total.

Save uses native dialogs and atomic replacement via a same-directory temporary
file. A file changed externally since open/save is not silently overwritten: use
Save As. Save/Discard/Cancel protects New, Open, demo replacement and window close.
Cmd/Ctrl-S saves, Shift-Cmd/Ctrl-S saves as, Cmd/Ctrl-O opens. Saving commits active
numeric draft values. File dialogs and shortcuts use the platform backend.

Recovery is written after a 700 ms debounce in the operating system's TinkerCraft
app-data directory. It is separate from saved files. Startup offers Restore or
Discard. Restore creates an unsaved copy with no overwrite target. Damaged recovery
is retained and automatic recovery writes are paused. Only one running instance
per user is supported: concurrent instances share the recovery location.
Recovery is not a backup; save important work explicitly. Previous browser/Tauri
recovery locations are not automatically imported; explicitly saved projects are.

Export STL writes the complete evaluated positive geometry in millimeter
coordinates, independent of selection. Export is disabled while computation is
pending or invalid.

Import STL accepts binary or single-solid ASCII files (10 MB / 20,000 triangles).
Choose source units (mm, inches, cm), validate, then review dimensions and add.
STL is unitless: millimeters are the default, not inferred. Imported objects are
centered at X40/Y15 and placed on Z0; original STL placement is not preserved.
The importer welds exactly matching vertices, corrects globally reversed winding,
and rejects degenerate/duplicate faces, open or non-manifold edges, invalid values,
and detected self-intersections. It does not repair holes or simplify meshes.
Imports run on a worker and can be cancelled; adding one is a single undo step.
Meshes support dimensions, transforms, mirroring, solid/hole cuts, grouping,
save/reopen and export. Mesh geometry is embedded, with no external file dependency.
Version 7 saves cannot be opened by older checkpoint builds.
You can also launch `tinkercraft path/to/model.stl` to open the same import review.

## Architecture

| Crate | Responsibility |
|---|---|
| `tinkercraft-core` | Document validation/migration, history, operations, Boolean engine, STL, storage, camera and rasterizer |
| `tinkercraft` | Native egui editor, dialogs, shortcuts, recovery and worker orchestration |
| `tinkercraft-cli` | Headless validation, STL import/export and viewport rendering using the same engine |

Geometry runs on one background worker. Queued edits coalesce; stale replies cannot
replace newer results. The native viewport uses a portable depth-tested **CPU
rasterizer**, presented as an egui texture through the native OpenGL backend.
Camera changes rerender it; idle frames reuse the texture. Resolution is capped
at 900 pixels per dimension. Holes use stippling instead of alpha transparency.
This is a deliberate first-port implementation, not a claim of GPU-native 3D
performance or retina-quality CAD rendering. Large scenes need profiling and may
benefit from replacing this renderer with a wgpu backend; the engine is isolated.

No ArtCraft code or brand assets are incorporated. Its engine-first/native design
informed the structure, while TinkerCraft retains its own document format and tools.

## CLI

```sh
cargo run --release -p tinkercraft-cli -- self-test
cargo run --release -p tinkercraft-cli -- import model.stl imported.tinkercraft mm
cargo run --release -p tinkercraft-cli -- demo example.tinkercraft
cargo run --release -p tinkercraft-cli -- validate example.tinkercraft
cargo run --release -p tinkercraft-cli -- export example.tinkercraft example.stl
cargo run --release -p tinkercraft-cli -- render example.tinkercraft example.ppm
```

`validate` checks both the project and its printable geometry; an empty/hole-only
project is editable but will return an error from this command. `render` produces
a PPM image without opening a window. The CLI is not an MCP server.

See CHECKPOINTS.md for this delivery, RELEASE.md for verification gates, and
ROADMAP.md for remaining modeling work.
See MIGRATION.md for port verification; see THIRD_PARTY.md for
licenses. The pure-Rust Manifold port is a different dependency from the former
C++/WASM library. Comparison tests validate representative fixtures, not every
possible pathological Boolean input.

## Native UI regression smoke test (Linux)

After building release binaries, install Xvfb and xdotool, then run
`python3 scripts/native-smoke.py`. It uses isolated app data and a 1440×960 display
at default UI scale. File-picker integration is separate; the import test launches
a file through the same validation/review flow. No user projects are modified.
