### JLCPCB CPL rotation rules: policy `konnect-jlcpcb-cpl-v2` (minor release)

The built-in CPL rotation corrections changed from 2 rules to 29: 24 backed by observed JLCPCB
placement previews, and 5 unverified (warned about, KiCad's rotation kept) (`docs/JLCPCB_CPL_CORRECTIONS.md`, "Built-in rules"). Footprints that now get a
correction where none applied before include every `SOIC-*`, `SSOP-*`, `MSOP-*`, `TQFN-*` and
`VQFN-*` (+270) and `SOT-23` / `SOT-23-5` (+180); the HRO USB-C rule (+180) was withdrawn and that
connector now keeps KiCad's rotation with a warning. A board exported again can therefore get a
different CPL file. The policy id in responses changed from `konnect-jlcpcb-cpl-v1` to
`konnect-jlcpcb-cpl-v2`, so the two rule sets can be told apart. Check the rotations of affected
parts in JLCPCB's preview before ordering, and pin any part with a designator override. The
review warnings now list every part a built-in rule rotated, and bottom-side parts with a non-zero
correction (every observation behind the rules is of a top-side part).
