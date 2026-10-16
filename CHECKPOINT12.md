# Checkpoint 12 — native builds verified

Completed 2026-10-16. Application version remains 0.11.0.

## Evidence

[Successful build run](https://github.com/partiizan/tinkercraft/actions/runs/38957322621)

Source commit: `8278f72e999c79c50b488c57695569d62de3e877`.

- macOS ARM64, Windows x64 (MSVC), Linux x64: native builds passed.
- Each platform: 37 engine tests, formatting, Clippy with warnings denied,
  four packaging tests, release compilation, package checksums/inventory,
  extracted CLI self-test and desktop version probe passed.
- Linux: native GUI regression passed.
- Initial Linux GUI run exposed a missing libxkbcommon-x11 runtime library;
  added libxkbcommon-x11-0 to CI and README prerequisites. Rerun passed.
- All 565 tracked checkpoint-11 files were transferred with matching Git blobs.
  Prior local checkpoint history remains in the checkpoint-11 delivery's Git bundle;
  GitHub history begins with this source import.

## Download and try on your Mac

1. Open the successful build run above while signed in to GitHub.
2. Under Artifacts, download **TinkerCraft-macOS-ARM64**.
3. Extract the artifact ZIP, then extract `TinkerCraft-Darwin-aarch64.zip` inside.
4. Move `TinkerCraft.app` to Applications and open it.

This is an unsigned, unnotarized development build. If macOS blocks opening it,
use its per-app approval flow only for this build you downloaded from your repo;
do not disable system-wide protections. No Rust installation is needed to run it.

Create a box and cylinder hole, group them, undo/redo, save/reopen through the OS
file dialogs, then export STL and check its dimensions in Bambu Studio. Also test
resize, snapping, keyboard shortcuts, hide/lock, preferences and recovery.
Report any error text or screenshot so failures can be reproduced.

Windows artifact: **TinkerCraft-Windows-X64**; extract the nested native ZIP and
run `tinkercraft.exe`. Linux artifact: **TinkerCraft-Linux-X64**; extract the
nested tar.gz and run `tinkercraft`, with README runtime dependencies installed.

## Remaining acceptance

CI package execution is verified. Mac/Windows graphical interaction, real OS file
pickers, slicer checks and an actual print remain unverified. See RELEASE.md for
the full checklist. These are portable development packages, not signed installers.
Artifacts follow GitHub retention settings; no permanent GitHub Release has been
published. The initial package documentation predates this CI result; repository
documentation is current. No modeling features changed in this checkpoint.
