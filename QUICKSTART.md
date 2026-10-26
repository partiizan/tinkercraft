# TinkerCraft quick start

## Install

Download the archive for your system from
https://github.com/partiizan/tinkercraft/releases and extract it fully.

- **Apple Silicon Mac:** move `TinkerCraft.app` into Applications. The bundle is
  ad-hoc signed, not Apple notarized. If macOS blocks the downloaded app, use the
  explicit Open Anyway option in System Settings → Privacy & Security after
  checking that you downloaded this repository's release. Do not disable Gatekeeper.
- **Windows x64:** open `tinkercraft.exe` in the extracted folder. The executable is
  unsigned; Windows may show a publisher warning. A modern OpenGL driver is required.
- **Linux x64:** run `./tinkercraft` from the extracted folder. Use an X11/XWayland
  desktop with OpenGL and a working XDG desktop portal/file-picker backend.
  See README.md for distribution dependencies.

The app runs locally without an account. Keep the included CLI and examples if
useful; they are not required alongside the Mac app. Check the adjacent `.sha256`
file to verify an archive's checksum.

## Make a two-hole plate

1. Choose New. Add a Box and set its dimensions to 80 × 30 × 5 mm.
2. Add two Cylinders, make each a Hole, and position them through the plate.
   Make each cylinder taller than the plate so it crosses both faces.
3. Select the plate and both holes, then Group (Cmd/Ctrl-G). The cut result appears
   automatically. The resulting group remains selectable, movable, rotatable and
   uniformly resizable. Group it with another hole to make another cut.
4. Ungroup (Shift-Cmd/Ctrl-G) to edit the original components. Undo restores the
   previous operation. Full export preview is read-only: choose Back to editing
   before making changes there.
5. Save the editable recipe as `.tinkercraft`. Export STL for geometry, or 3MF for
   separate colored material parts. Saving a project is separate from exporting.

Dimensions are in millimeters and positions refer to centers. Use Drop to workplane
when a resize places the bottom below Z=0. Hidden objects still participate in
cuts and exports; hiding is only a viewport convenience.

## Bambu Studio

Import the STL or import the 3MF as a model, keeping its parts together. Select
X1 Carbon and your actual nozzle/filament profiles in Bambu Studio, check dimensions,
assign each part to a filament, and inspect the sliced preview before printing.
TinkerCraft does not send jobs to a printer or generate G-code.

The two-color fixture imports in Bambu Studio 02.08.02.61 at 80 × 30 × 5 mm;
STL is one manifold mesh and 3MF has two manifold parts. Generic 3MF RGB materials
are written correctly, but Bambu's CLI round-trip did not preserve these as filament
colors. Assign filaments manually; automatic AMS/RGB mapping is not included.
An actual print and Bambu's interactive filament-assignment flow have not been tested.

## Controls and preferences

Orbit by dragging the background, right-drag to pan, and scroll to zoom. W/E/R
select Move/Rotate/Resize. Arrow keys move in X/Y, Page Up/Down in Z, and Shift
multiplies the step by ten. F fits the selection. Cmd/Ctrl-D duplicates.
Preferences includes Light, Dark and System appearance. The inspector's Color
control recolors a solid; recoloring a group recolors all of its components.

Projects include a recovery snapshot. After an interrupted session, choose Restore
at startup. Recovery is not a substitute for saving your project. Old app versions
cannot open newly saved schema-8 files; preserve originals when upgrading.

New builds are release candidates marked **Pre-release** on GitHub. Select the
newest candidate from the Releases list; GitHub's `/latest` URL points to the last
stable release instead. Remaining acceptance: https://github.com/partiizan/tinkercraft/issues/3.

File shortcuts: Cmd/Ctrl-I imports STL; Cmd/Ctrl-Shift-E exports STL;
Cmd/Ctrl-Alt-E exports 3MF. Use ACCEPTANCE.md to report an RC test.

## Additional modeling tools

Press M for measurements, a two-point ruler, distribution, linear/radial copies
and curve-resolution presets. Select 3+ objects to distribute; end objects stay
fixed. Select objects then Create copies; Additional copies excludes the originals.
A full radial sweep closes the circle without duplicating its start.

Select a group and press F2 to edit individual components, including holes, without
ungrouping. Positions and rotations are relative to the containing group. Apply
changes rebuilds cuts in one undo step; Cancel discards the staged changes.

The ruler can use numeric XYZ points or two snapped clicks on Z=0. Press Escape to
cancel picking. Selection dimensions and gaps use source bounding boxes before
cuts, not exact curved-surface distances. Default curve resolution remains 96.

## Shapes, labels and selection workflows

Use More shapes in the left panel for Tube, Torus, Polygon or Text. Change bore
ratio, polygon sides or lettering in the inspector. Text supports A–Z, digits,
spaces and - . _ +; its width fits the full label. New lettering starts slightly embedded in the top
of the selected solid’s source bounds (otherwise on Z=0). Make lettering a Hole and overlap
it with a solid to engrave, or leave it Solid and overlap a base for raised text.
Group to apply the result. The modeling-catalog example shows both approaches.

Press M and expand Transform selection together to move, rotate and uniformly
scale multiple objects without grouping. Apply uses one undo step. Expand Align
to reference object to align selected objects to another object's bounds; choose
each edge and an offset. The reference stays fixed, including when locked.

Open Object browser from the left panel to search names/types/nested components,
inspect group contents, select matches, invert selection or rename objects. Batch
names receive numbered suffixes. Use F2 on a selected group to edit components.

## Modeling studio · F4

Use a numeric origin/rotation or Pick a face for a custom workplane. Apply plane
sets it, Reset XY restores the original plane, and Look at workplane faces its
normal. New shapes, drop-to-workplane and ruler picks use the active plane; object
coordinates remain world XYZ. The plane is stored with the project.

For object snapping, select objects, choose Pick source, then target, click the
selection, then another object's surface. Toggle triangle-vertex snapping if
needed. Escape cancels. Shift-drag adds objects by their projected bounds centers;
Alt-Shift-drag removes them. 1/2/3/4 choose top/front/right/isometric camera views.

F3 enters a selected group in a local-coordinate workspace. Use regular handles
or numeric controls on children; F3 again enters a nested group. Shift-F3 applies
and exits. Cancel group edits discards the active workspace. Save/export apply all
active workspaces first. Recovery stores the full scene, even while inside a group.

Choose SVG or TTF/OTF in the studio, set width/depth, then generate. Review the
reported dimensions and Add generated mesh. Plain SVG filled paths, strokes and
cutouts are supported; convert SVG text/clips/effects to paths first. Font lettering
supports Latin glyph outlines; regenerate to change font/text. The resulting meshes
are self-contained and support the normal solid/hole/color/group/export workflows.
You can also launch TinkerCraft with an SVG or font path to open this import panel.

The studio has GPU/software rendering and interface-scale controls. F1 opens the
keyboard reference. Normal controls support Tab/Shift-Tab and Space/Enter, with
native accessibility integration. Use PARITY-ACCEPTANCE.md for remaining manual
checks before stable-release or scoped-parity sign-off.
