# Roadmap after the Rust migration

These are scope estimates, not token or weekly-usage estimates. A checkpoint is a
bounded implementation/test/delivery milestone; effort varies substantially.

## MVP release candidate after checkpoint 11

Implementation checkpoints 10 and 11 deliver visibility/locks, saved preferences,
keyboard movement, group drop, fit-selection and verified release packaging.

Checkpoint 12 runs native builds and packaged CLI checks in GitHub Actions.
Remaining acceptance gate: Mac and Windows interactive GUI checks, plus real
OS file-dialog and slicer-export checks on all targets. See CHECKPOINT12.md for
CI evidence; GUI acceptance still needs target desktops. Reserve one or two fix checkpoints if acceptance finds bugs.
Do not describe compile-only checks as a completed cross-platform release.

## Broad 3D-editor parity: approximately 12–20 further checkpoints after MVP

- Custom workplanes, rulers and measurement.
- Text and SVG import/extrusion.
- Larger primitive and shape catalog; configurable tessellation.
- Editing within nested groups, richer transform/selection controls.
- More complete snapping, alignment/distribution and camera behavior.
- Object appearance, scene organization, and interaction polish.
- GPU viewport work and larger-scene performance.
- Accessibility, keyboard navigation, regression coverage and release hardening.

Circuits and code blocks remain explicitly out of scope. These estimates assume
local 3D-editor parity, not Tinkercad's cloud storage, sharing/collaboration or
classroom ecosystem. Those require a separate product/backend/security effort.

A detailed feature inventory and acceptance tests are needed before promising
literal feature parity. No completion date or weekly-usage cost is implied.

## Scope clarified by the user

Parity refers only to interface functions directly used to model and manage simple
objects for local 3D printing. Prioritize dimensioning, placement, combining solids
and holes, organization, measurement and printable export. It does not mean
replicating the full Tinkercad site, content ecosystem, social features or branding.
The 12–20 parity-checkpoint range remains provisional pending a modeling-only audit.
