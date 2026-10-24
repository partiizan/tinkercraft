# Checkpoint 19 — Bambu workflow acceptance (partial)

Extended scripts/validate_slicer.py from import checks to actual slicing with the
official Bambu Studio 02.08.02.61 Linux build. The checked-in result is
validation/checkpoint19-bambu.json. This is a local CLI test with the 0.17.0 source
engine, not interactive UI or physical printer evidence.

The 80 × 30 × 5 mm fixture imports as manifold geometry. Both STL and single-filament
3MF slice to 25 layers. A Bambu-created project with its two parts programmatically
assigned to filament slots 1 and 2 also slices successfully: both filaments have
positive model usage and there are 25 filament changes, with no reported slicing
warnings. Generic RGB-to-AMS mapping is still not provided by TinkerCraft.

Regression settings: X1 Carbon 0.4 mm nozzle, 0.20 mm Standard process, Generic PLA.
The script resolves the bundled profiles' inheritance and included G-code templates;
loading raw preset fragments alone does not reproduce these settings. These are
explicit test settings, not an assertion about the user's installed nozzle/plate
or filament calibration. Generated G-code is temporary and deleted, never sent to
a printer or included in the release.

Reproduce with a native Bambu executable and its bundled BBL profile directory:

```sh
python3 scripts/validate_slicer.py /path/to/bambu-studio \
  --profiles /path/to/resources/profiles/BBL --report slicer-results.json
```

Remaining: interactive Bambu part/filament assignment, slicing with the user's
actual profiles, and an X1C print with measured dimensions/holes/color boundaries.
Record that evidence in ACCEPTANCE.md and issues #2/#3. This checkpoint is not
marked fully complete until those hands-on checks have been performed.
