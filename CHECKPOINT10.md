# Checkpoint 10 — visibility, locks and local preferences

- Project schema 7 stores hidden/locked flags and reads schemas 1–6.
- Hide is viewport-only: it does not suppress cuts or remove anything from STL.
  Result preview/export always contains all objects. Show all restores visibility.
- Locks protect modeling edits and deletion at the history boundary, including
  compound/group actions. A locked child protects its top-level group. Unlock
  recursively unlocks the group. Visibility toggles and undo/redo remain available.
- Hidden/locked objects are omitted from Select all. They can still be selected
  explicitly in the full-width object list to show/unlock them.
- Local snap, tool, projection and optional camera preferences persist separately
  from project contents, with bounded validation and atomic writes. An unreadable
  preferences file is retained until Reset defaults is explicitly selected.
- Hide/lock changes are undoable and are saved in project/recovery files.
- Six regression scenarios cover visibility/export invariance, inherited hiding,
  lock bypass prevention, group locks, preference round trips and invalid files.
