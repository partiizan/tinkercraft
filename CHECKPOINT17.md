# Checkpoint 17 — MVP distribution and handoff

Version 0.16.0 completes the planned implementation/release work for the local
modeling MVP, conditional on the native CI release gates passing.

- Packages include version, operating system and architecture in their filenames.
- Mac signing remains after all bundle resources are assembled; extracted bundles
  are verified again. No Apple Developer ID or notarization credentials are present.
- Every archive includes a per-file checksum inventory; a separate SHA-256 file
  identifies the archive.
- A release job publishes the three platform archives and checksums only after all
  native jobs succeed. Existing versions are never replaced automatically.
- QUICKSTART.md covers installation, editable holes/groups, recovery, exports and
  Bambu filament assignment. RELEASE-NOTES.md records the supported MVP scope and
  remaining manual acceptance limits.

No additional feature checkpoint is needed for this MVP's agreed scope. Remaining
acceptance tasks are hands-on Mac/Windows dialogs/shortcuts, varied graphics drivers,
and a real printer trial. Those may reveal follow-up bugs; they are not silently
marked tested. Modeling parity work remains a separate roadmap.
