# Checkpoint 14 — appearance preferences (0.14.0)

Preferences now offers Light, Dark and System. System is the default for new and
legacy preferences. The egui/winit native backend reports OS appearance changes;
when the platform does not report a theme, System falls back to Dark.

UI controls, viewport background/grid and viewport labels follow the resolved
theme. Changes are immediate, saved separately from projects, and do not change
geometry or undo history. Reset defaults restores System.

Core coverage: legacy preference migration, all three modes surviving restart,
and viewport theme changes retaining the same picking behavior. Native platform
and UI validation is recorded with checkpoint 15's combined build.
