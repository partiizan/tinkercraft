# Checkpoint 13 — modeling feedback and app identity

Application version 0.13.0. Changes requested after the first Mac launch:

- Grouping solids and holes now automatically displays the Boolean result.
  Both Group and Cmd/Ctrl-G use the same path. Failed operations do not switch
  modes. Ungroup and Undo restore source editing; grouping remains nondestructive.
- Primitive palette buttons display a name above a small matching illustration.
- Matte charcoal toolbox artwork with teal/orange screwdrivers is used for the
  native window icon and macOS application bundle. Windows packages also include
  an ICO for shortcuts; EXE resource embedding is not part of this checkpoint.
- Build packaging signs the fully assembled macOS bundle with an ad-hoc signature,
  including its icon/resources, before writing the checksum manifest. CI verifies
  both staged and extracted signatures. This addresses the reported missing
  signature-resources error; it does not provide notarization or Developer ID trust.

Native build results will be recorded after validation. Manual testing of the new
Mac package, OS file dialogs and Bambu Studio export remains required.
