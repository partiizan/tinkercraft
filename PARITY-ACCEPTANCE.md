# Final manual acceptance and modeling parity audit

Status: feature implementation complete for the planned local-modeling scope;
**manual acceptance and full feature-parity confirmation are still open**.
Record the exact RC version, OS, graphics driver, input device, expected result,
actual result and project file for each failure. Do not sign off a newer build
using only results from an older candidate. Track findings in GitHub issue #3.

## Scoped feature inventory

- Primitive/parametric shapes: box, cylinder, sphere, cone, wedge, roof, tube,
  torus, polygon prism and editable block lettering; curve-resolution presets.
- Local/world modeling: move/rotate/resize, independent selection transforms,
  grid/angle snap, point-to-point snap, alignment/reference alignment, distribution,
  mirroring, linear/radial copies, custom workplanes, ruler and dimensions.
- Objects and Boolean modeling: solid/hole subtraction, nested editable groups,
  numeric and viewport child editing, hide/lock, search/select/batch naming, colors.
- Interchange: bounded STL import, SVG extrusion, font-outline extrusion, STL and
  colored 3MF export, saved projects, migrations, atomic saves and recovery.
- Native interaction: camera views/orbit/pan/zoom, box selection, keyboard commands,
  accessibility integration, UI scaling/themes, GPU rendering and software fallback.

## Remaining manual checks — all unchecked

- [ ] Install and launch the same RC on real Apple Silicon macOS, Windows x64 and
  Linux x64. Test a clean profile and an upgrade from the previous release. Confirm
  OS prompts, packaged icon, file pickers, writable paths and normal shutdown.
  Mac signing is ad-hoc, not Developer ID/notarized; Windows is unsigned.
- [ ] Recreate a practical part from scratch: dimensions, rotations, snapping,
  duplicate/array, align/distribute, mirror and independent multi-object transforms.
  Verify numerical values, constrained circular dimensions and undo/redo after each
  operation. Confirm each advertised primitive and curve-resolution setting.
- [ ] Reproduce the original two-hole regression: group a rectangular plate and
  two holes, resize/move the result, add another cut, enter the group with F3, edit
  children with handles, enter a nested group, cancel and apply, then undo/redo.
  Confirm hierarchy, parent transforms, names, IDs, colors and locks survive.
- [ ] Test workplanes on flat and rotated faces, numeric plane placement, new-shape
  orientation, drop-to-plane, plane-aligned view and ruler picks. Verify source and
  target snapping for vertices/surfaces and grouped objects; Escape cancels cleanly.
- [ ] Import representative STL, SVG logos with holes/transforms/strokes and your
  own TTF/OTF fonts. Compare outline appearance, counter holes, dimensions and
  extrusion depth to the source. Confirm unsupported/oversized files produce clear
  errors. Save/reopen on another computer without the original SVG/font files.
- [ ] Exercise persistence: Save/Open/Save As and cancel dialogs, overwrite/external
  changes, non-ASCII filenames, permission errors, legacy project opening, and
  forced-close recovery. Save and export while inside nested group editing; verify
  the full project is recovered/exported rather than only its current children.
- [ ] Check GPU/software visual agreement for geometry, holes and colors; high-DPI
  and multiple monitors; trackpad/mouse orbit/pan/zoom; themes and UI scaling. Use
  dense projects approaching the documented limits to check responsiveness, memory,
  cancellation and handling of geometry failures.
- [ ] Test keyboard-only navigation and VoiceOver/Narrator/Orca: discoverable control
  names, focus order/visibility, dialog dismissal, numeric editing, shortcuts and
  alternatives to handle dragging. Test box selection and hidden/locked exclusion.
- [ ] Open exported STL and multicolor 3MF in Bambu Studio for the X1C. Confirm mm
  dimensions, manifold geometry, cutouts and component/color boundaries. Assign
  actual filaments/AMS slots interactively and review slicing and layer previews.
  Generic RGB 3MF colors do not automatically identify your physical filaments.
- [ ] Print a calibration part with two holes, a boss, raised and recessed text,
  and multiple colors if available. Measure dimensions and hole fit with calipers;
  distinguish source dimensions from slicer compensation/printer tolerances. Check
  that Boolean seams, lettering counters and thin features print as intended.
- [ ] Perform a side-by-side modeling workflow audit against the Tinkercad functions
  you actually require. Every required workflow must be present and behave correctly;
  record any missing capability as feature work, not a passed manual check. Explicitly
  accept the limits below before calling the result complete for this scoped use.
- [ ] Resolve all blocking findings, rerun affected checks on the final RC, and give
  explicit approval to publish a stable release. Issue closure never promotes it
  automatically. Checkpoints 18–20 remain these acceptance/release gates.

## Known limits that manual testing cannot remove

- Project limits: 100 nodes, eight group levels, 20,000 imported/generated mesh
  triangles and 32 MB project files; dimensions 0.01–1000 mm and centers ±10000 mm.
- Groups resize uniformly; no nonuniform affine/shear editing of a group hierarchy.
- Measurements/alignment use source bounding boxes before cuts; point snaps use the
  tessellated mesh. These are not exact analytic-surface measurements or constraints.
- SVG is monochrome vector geometry. Convert raster/text/clipping/masks/effects to
  plain paths before import; external resources are not fetched.
- Font lettering supports Latin outlines/legacy kerning; complex scripts/ligatures
  require pre-outlined SVG. Generated lettering is a mesh and must be regenerated
  to change its text/font. Parametric block text uses the built-in limited alphabet.
- GPU work does not promise a frame-rate on every driver. Software fallback remains
  available. Screen-reader integration needs real assistive-technology acceptance.
- No circuits, code blocks, cloud collaboration/accounts, classrooms or replication
  of Tinkercad's complete community shape/generator catalog.

The planned checkpoint count is not evidence of literal one-for-one parity.
Acceptance establishes the agreed simple local 3D-printing workflows; a broader
parity claim requires expanding and passing the feature inventory above.
