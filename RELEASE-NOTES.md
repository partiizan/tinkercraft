This is a **release candidate**, not a stable release. Each successful main-branch
build receives a unique `0.17.0-rc.N` version; the tag and downloads identify the
exact candidate. The number is the workflow run number, not a completed-checkpoint
count. Rerunning a workflow retains its candidate identity across all platforms.

Track remaining acceptance and report candidate-specific results in
https://github.com/partiizan/tinkercraft/issues/3.

Remaining gates: native desktop dialogs/shortcuts on user hardware, interactive
Bambu filament assignment and a physical print, followed by an explicit stable
release decision. Closing issues never automatically promotes a build to stable.

TinkerCraft 0.16.0 completes the implementation and automated validation checkpoints
for the local modeling MVP. Rust-native packages target Apple Silicon macOS,
Windows x64 and Linux x64. Circuits, code blocks and cloud features are out of scope.

- Editable Boolean groups: solids and holes cut automatically, and the resulting
  group supports further transforms, cuts, grouping and undo.
- Primitive modeling, STL import, numeric/handle transforms, snapping, alignment,
  mirroring, visibility/locks, saved projects and recovery.
- Light/Dark/System themes, object colors and generic color-part 3MF export.
- Quitting waits for an active 3MF export instead of interrupting its worker.
- Versioned downloads, per-file manifests and archive SHA-256 checksums.

Publishing is gated on all three native build/test/package jobs, including opening
an extracted packaged app and checking its rendered framebuffer. Linux also runs
mouse/keyboard modeling and recovery regressions. Official Bambu Studio 02.08.02.61
imports the STL and 3MF fixture with correct dimensions and manifold geometry.

Read QUICKSTART.md inside the archive. Mac bundles are ad-hoc signed and verified,
not Developer ID signed/notarized; Windows executables are unsigned. These are
portable packages, not installers. A modern OpenGL driver is required.

Hands-on Mac/Windows file-dialog/shortcut acceptance and physical printing remain
unverified. Generic RGB materials are not an automatic Bambu filament/AMS mapping:
Bambu's CLI preserves the parts but drops their RGB materials on re-export. Assign
filaments in the slicer. This release does not claim full Tinkercad modeling parity.

Windows hosted-runner graphics use the checksum-pinned Mesa 26.2.4 software
OpenGL driver for the extracted-window test, because the Hyper-V display adapter
only exposes legacy OpenGL. These test-only DLLs are not shipped in the archive.
The check verifies the native Windows application/window/rendering path, not
compatibility with every physical GPU. Startup failures now display a useful
error dialog instead of silently exiting.
