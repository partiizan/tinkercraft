# Roadmap after the Rust migration

## Current release-candidate policy

New main-branch builds publish as GitHub **Pre-releases**, versioned `0.20.0-rc.N`,
after all three native jobs pass. Manual workflow runs on main do the same;
pull requests and failed builds do not publish. A rerun keeps the same candidate
number so partial retries cannot mix different platform versions. Already published
assets are never replaced. The existing v0.16.0 release remains historical.

[Issue #3](https://github.com/partiizan/tinkercraft/issues/3) tracks checkpoints
18 (desktop acceptance), 19 (Bambu/printing acceptance, including #2), and 20
(explicit stable-release approval). The items below describe prior implementation
milestones; they do not imply that those manual acceptance gates have passed.
There is no automatic stable-release promotion in the workflow.


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

## Checkpoints 21–25 · modeling tools

Measurement/ruler tools, nested component editing, distribution, repeated copies
and curve resolution are implemented together in the 0.18 RC series. See
CHECKPOINT21-25.md for limits and validation. Hands-on acceptance is deferred until
this batch is delivered; checkpoints 18–20 remain open acceptance/release gates.

## Checkpoints 26–30 · shape and selection tools

Tube/torus/polygon shapes, editable block lettering, reference-object alignment,
independent multi-object transforms and a searchable object browser are delivered
in the 0.19 RC series. See CHECKPOINT26-30.md for limits and validation. More
complete workplanes, SVG/custom-font extrusion, viewport child manipulation, GPU
rendering and further interaction polish remain future modeling work. Hands-on
acceptance remains deferred until feature work is finished.

## Checkpoints 31–38 · planned feature implementation complete

Custom workplanes, SVG/font extrusion, viewport group editing, point snapping,
box selection/navigation/accessibility, GPU rendering and release hardening are
implemented. CHECKPOINT31-38.md defines the scope and limits. No further feature
checkpoint is scheduled in this agreed batch. PARITY-ACCEPTANCE.md lists the open
manual tests and workflow audit. New requirements or defects from that audit can
require additional feature/fix candidates; literal Tinkercad-wide parity is not
asserted. Checkpoints 18–20 remain acceptance and explicit stable-release gates.
