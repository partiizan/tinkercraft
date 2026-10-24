# Checkpoint 16 — MVP validation and export shutdown safety

Version 0.16.0, project schema remains 8.

Added an isolated packaged-window smoke mode (`--smoke-test REPORT.json`) which
requires a real OpenGL context, renders the demo geometry, requests an eframe
framebuffer screenshot, checks nonempty model pixels and a nontrivial window image,
then exits. It does not read or write user preferences or recovery. The native
package verifier runs this mode outside the repository with a 90-second timeout.
All three native CI jobs now require it. This is a graphics startup check, not
proof of every OS dialog, shortcut, graphics driver or monitor configuration.

Fixed closing the application during asynchronous 3MF export: close is cancelled
with a status message until export finishes; errors are still reported and the
user may then close normally. OS force-quit cannot be prevented.

Local Linux verification passed: 46 core tests, four packaging tests, strict
lib3mf validation, native mouse/keyboard regression and extracted window smoke.
The window probe captured 1440 × 960 pixels, 4,168 distinct colors and 111,974 model
pixels. See CI for independent native Mac/Windows execution on the release commit.

Bambu Studio's official Linux build 02.08.02.61 imported both exports of
`examples/multicolor-plate.tinkercraft`. Both measured 80 × 30 × 5 mm and were
reported manifold. STL had one part, 796 facets, volume 11803.786133 mm³. 3MF had
two parts, 800 facets, volume 11803.783203 mm³ (expected 11803.790612 mm³).
`scripts/validate_slicer.py PATH_TO_BAMBU` reproduces these assertions.

Bambu's CLI 3MF re-export retained the two parts but did not retain generic RGB
materials as filament assignments. Manual filament selection remains required.
The exporter remains a standards-based model package, not a printer-specific job.
No G-code, physical print or interactive filament-mapping result is claimed.

Windows hosted-runner graphics use the checksum-pinned Mesa 26.2.4 software
OpenGL driver for the extracted-window test, because the Hyper-V display adapter
only exposes legacy OpenGL. These test-only DLLs are not shipped in the archive.
The check verifies the native Windows application/window/rendering path, not
compatibility with every physical GPU. Startup failures now display a useful
error dialog instead of silently exiting.
