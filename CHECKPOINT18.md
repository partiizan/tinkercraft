# Checkpoint 18 — native desktop acceptance (partial)

A new Linux CI gate exercises real GTK portal file dialogs on an isolated D-Bus
session and X display. It groups the demo plate and holes, saves, nudges and saves
again, opens changed project content, uses Save As and cancellation, exports STL
and 3MF, and opens the STL import picker. Existing native regression covers the
import review/add flow and grouped transforms/undo/recovery. This must pass before
an RC is published; consult its workflow run for execution evidence.

Added consistent file-workflow keyboard shortcuts (Cmd on Mac, Ctrl elsewhere):
I imports STL, Shift-E exports STL, Alt-E exports 3MF. Existing S/Shift-S/O handle
Save/Save As/Open. The shortcuts reuse the same picker/validation/export paths as
the buttons. Active transform drags still intercept shortcuts until committed or
cancelled, and exports commit the current draft before generating output.

Mac/Windows native-window checks remain CI gates, but physical hardware, real
file pickers, shortcut behavior, theme switching and scaling still need hands-on
acceptance. Linux GTK CI is not certification of every desktop/portal backend.
Those items stay open in issue #3. Use ACCEPTANCE.md to record results.

## Verified RC16 automation

[Run 40145152773](https://github.com/partiizan/tinkercraft/actions/runs/40145152773)
passed all three native jobs and published
[0.17.0-rc.16](https://github.com/partiizan/tinkercraft/releases/tag/v0.17.0-rc.16).
Linux passed the full GTK portal Save/Open/Save As/cancel/import/export sequence,
including saving and reopening an edited cut group. Native picker titles are now
explicit. Source commit: `bd706c9d5d40b2819f171c407c2e5583804bb500`.
This closes the automated portion, not the physical-device checklist above.
