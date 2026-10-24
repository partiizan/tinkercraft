# Roadmap after the Rust migration

These are scope estimates, not token or weekly-usage estimates. A checkpoint is a
bounded implementation/test/delivery milestone; effort varies substantially.

## MVP implementation complete in 0.16.0

Checkpoints 16–17 complete packaged native-window verification, Bambu Studio model
import validation, export shutdown safety, versioned releases and the quick start.
Release publication requires the native Mac, Windows and Linux CI jobs to pass.
Hands-on OS file-dialog/shortcut acceptance and physical printing remain unverified;
see RELEASE.md. Any bugs found there are maintenance work, not new feature scope.

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
