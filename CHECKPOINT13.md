# Checkpoint 13 — modeling feedback and app identity

Application version 0.13.0. Changes requested after the first Mac launch:

- Grouping solids and holes now automatically displays the Boolean result.
  Both Group and Cmd/Ctrl-G use the same path. Failed operations do not switch
  modes. Ungroup and Undo restore source editing; grouping remains nondestructive.
- Primitive palette buttons display a name above a small matching illustration.
- Matte charcoal toolbox artwork with teal/orange screwdrivers is used for the
  native window icon and macOS application bundle. Windows packages also include
  an ICO for shortcuts; EXE resource embedding is not part of this checkpoint.
- Build packaging signs the fully assembled macOS bundle with an ad-hoc signature,
  including its icon/resources, before writing the checksum manifest. CI verifies
  both staged and extracted signatures. This addresses the reported missing
  signature-resources error; it does not provide notarization or Developer ID trust.

## Validation

[Successful native run](https://github.com/partiizan/tinkercraft/actions/runs/38960079249)
for source `f6ed178bceff8007d2fec896a186369e1b61fce4`.

All three platforms passed 37 engine tests, Clippy, formatting, four packaging
tests, release builds and extracted package probes. Linux GUI regression passed.
Local visual inspection confirmed the illustrated palette and immediate cut result
for both button and keyboard grouping. macOS codesign reported `valid on disk`
and `satisfies its Designated Requirement` for staged and extracted bundles.

Downloads: [Mac ARM64](https://github.com/partiizan/tinkercraft/actions/runs/38960079249/artifacts/12072940197),
[Windows x64](https://github.com/partiizan/tinkercraft/actions/runs/38960079249/artifacts/12072516821),
[Linux x64](https://github.com/partiizan/tinkercraft/actions/runs/38960079249/artifacts/12072202995).
Extract the workflow ZIP and then the native archive inside. Replace the previous
app with this version (0.13.0). Manual Mac GUI testing, OS file dialogs and Bambu
Studio export remain required. The bundled checkpoint notes predate CI completion;
this repository records the final results.

## 0.13.1 grouped-object editing fix

The initial automatic cut display switched into the whole-scene preview, which
had no object picking or transform handles. Groups now render their own Boolean
geometry in the regular editing viewport, retaining their IDs and handles.
Move, rotate, uniform resize, add another solid/hole and regroup remain available.
Ungroup exposes the original components. The explicit Preview full export command
is still a whole-scene inspection mode; Back to editing exits it.

New regression coverage checks selection through the cut mesh, empty hole picking,
transformed/resized cut geometry, additional nested cuts, save/reload and undo.
The native GUI regression now groups the demo plate, drags it, resizes it, undoes
both operations and ungroups it.

Validation: all three native jobs passed in
[run 38975882427](https://github.com/partiizan/tinkercraft/actions/runs/38975882427)
for source commit 40ae6a0. All 40 engine tests and packaging checks passed, as did
Mac signature verification and the Linux grouped move/resize UI regression.
Manual testing of this updated Mac package remains open.
