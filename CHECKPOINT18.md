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
