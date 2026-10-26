# Release-candidate acceptance record

Copy this checklist into [issue #3](https://github.com/partiizan/tinkercraft/issues/3)
with results for the exact RC you downloaded. Unchecked means unverified.

Candidate tag: ___
OS/version: ___
GPU/driver (Windows/Linux): ___
Display scaling: ___

## Desktop

- [ ] Download/extract/install and launch without the source tree.
- [ ] Create a box with two overlapping hole cylinders; Group cuts automatically.
- [ ] Move/rotate/resize that group; group it with a third hole; undo/redo.
- [ ] Save using the native dialog, close, reopen and verify dimensions/cuts.
- [ ] Save As creates a separate project; cancelling a dialog preserves the original.
- [ ] Import STL, choose units, review/add, and undo the import.
- [ ] Export STL and 3MF through native dialogs and confirm output files.
- [ ] Test Cmd/Ctrl-S, Shift-S, O, I, Shift-E (STL), Alt-E (3MF).
- [ ] Switch Light/Dark/System, restart, verify preference persistence.
- [ ] Recover an interrupted unsaved edit; check scaled display and readable controls.

## Bambu/X1C

Bambu version: ___  Nozzle: ___  Plate: ___  Filament: ___
Process profile/layer height: ___

Use the included multicolor-plate.tinkercraft example. Open its STL and 3MF exports
in Bambu Studio as models. Keep the 3MF's two parts together and manually assign
filaments. Expected model envelope: 80 × 30 × 5 mm, with two nominal 5 mm through-holes.

- [ ] Slicer dimensions and two through-holes are correct.
- [ ] The two 3MF parts can be assigned distinct filaments interactively.
- [ ] Slice with the actual printer settings and inspect layer/color previews.
- [ ] Print and record measured X/Y/Z, both hole dimensions, color boundary and any defects.

Measured results / photo / notes: ___

Distinguish printer/material tolerance from exported geometry errors. The automated
slicer fixture uses X1C/0.4 mm/Generic PLA settings and does not establish your setup.
No app-generated files are sent to the printer automatically.

## Outcome

- [ ] No release-blocking failures remain (link any failures and reproduction steps).
- [ ] Explicit stable-release approval recorded after the remaining platform/print evidence.

CI success is recorded separately; do not check hardware/print boxes from CI alone.

## Final feature-batch handoff

The planned feature checkpoints through 38 are implemented. All hands-on checks
remain open; use PARITY-ACCEPTANCE.md as the consolidated checklist. It includes
real-device/assistive-technology testing, full-scene recovery, outline fidelity,
GPU/software behavior, Bambu filament mapping, a measured physical print, and a
side-by-side audit of required modeling workflows. Known product limits must be
accepted explicitly for a scoped parity claim; passing tests cannot remove them.
