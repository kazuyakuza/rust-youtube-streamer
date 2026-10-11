# fonts/

## Exo2-Light.otf (vendored test fixture)

- File: `Exo2-Light.otf` (111052 bytes, OpenType/CFF, magic `OTTO`)
- Full legal name: "Exo 2" Light
- Designer: Natanael Gama
- Source URL: `https://github.com/alexheretic/ab-glyph/raw/main/dev/fonts/Exo2-Light.otf`
- Downloaded (UTC): 2026-10-11 (performed inside the Compose `rust` container per plan amendment A7)
- SHA-256: `a4bbe688531db7853383649e17b67d7ded1d3a2492f5c3f44f715a6a3919b9ac`
- License: SIL Open Font License 1.1 — redistribution and embedding permitted; the full
  license text ships with the font in this directory (`OFL.txt`).
- Role: deterministic unit-test fixture for the renderer. Unit tests reference this file
  only (never `console.ttf`) via `concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/Exo2-Light.otf")`.

## OFL.txt (license text)

- File: `OFL.txt` (4386 bytes) — complete SIL Open Font License 1.1 text as shipped with
  the Exo 2 family (header: "Copyright 2013 The Exo 2 Project Authors",
  `https://github.com/googlefonts/Exo-2.0`).
- Source URL fetched: `https://raw.githubusercontent.com/google/fonts/main/ofl/exo2/OFL.txt`
- Downloaded (UTC): 2026-10-11
- SHA-256: `7f95a8eeecc6904ffc81cd10cc8ebc7fcee72a67922924bf45f02dff03b88127`
- Provenance note: the canonical upstream URL from the plan
  (`https://raw.githubusercontent.com/alexheretic/ab-glyph/main/dev/fonts/Exo2-Light.otf.LICENSE`)
  returned HTTP 404 on 2026-10-11, and the `alexheretic/ab-glyph` repository contains no
  OFL copy anywhere (root `LICENSE` is Apache-2.0 for the crate itself). The Exo 2 family
  OFL text from Google Fonts was used as the official fallback copy.

## console.ttf (pre-existing operator asset)

`fonts/console.ttf` predates this phase, is tracked, and its provenance is indeterminate.
It is not referenced by any test and must not be staged, modified, or deleted as part of
renderer work. Operators should supply their own documented font: the configured runtime
font path is operator-supplied at runtime and is checked by the renderer at construction
(file open and font parse) with typed errors; configuration validation performs no
file-existence check (Phase 01 precedent). Full documentation treatment happens in the
Task 3 docs scope.
