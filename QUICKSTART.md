# TinkerCraft quick start

## Install

Download the archive for your system from
https://github.com/partiizan/tinkercraft/releases/latest and extract it fully.

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
