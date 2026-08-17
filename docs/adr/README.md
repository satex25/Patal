# Architecture Decision Records

These live here, beside the code they constrain, rather than in the Obsidian vault
where they started. README and the memorandum both cite them normatively, and a fresh
cloner could not obtain them — a rule you cannot read is not a rule.

| # | Decision | Status |
|---|---|---|
| [001](ADR-001-stack-selection.md) | Rust core, SwiftUI shell, Metal, uniffi bridge | Accepted |
| [002](ADR-002-naming-convention.md) | `Pātāl` in prose, `Patal` for toolchains; bundle identifiers | Accepted |
| [003](ADR-003-curve-representation.md) | Curves as a layer above the polygon kernel, not inside it | Accepted |
| [004](ADR-004-document-format.md) | Schema version and material identity | Accepted |
| [005](ADR-005-tauri-as-engineering-harness.md) | Tauri unfrozen as a non-shipping harness | Accepted |
| [008](ADR-008-export-format-decisions.md) | Tiled PDF first; true scale, stroke centre, overlap-don't-trim | Accepted — partial, DXF still open |

006 and 007 are skipped deliberately rather than renumbered. Both are reserved by the
wedge-and-validation-wave blueprint, which cites them by number in a dozen places;
handing those numbers to something else would silently repoint every reference.

Not yet written, and named here so the gap is visible rather than forgotten:

- **ADR-006 — the competitive wedge.** There is no competitive analysis anywhere in
  this project. Seamly2D/Valentina is free, open source, parametric, roughly thirteen
  years old, and ships DXF-AAMA/ASTM export and tiled PDF printing. On the axis Pātāl
  currently competes on — draw a polygon, offset a seam allowance — it is behind a
  free incumbent. This should be written *after* actually drafting one bodice block in
  Seamly2D and Freesewing, not from a feature table.
- **ADR-007 — what a `PatternPiece` stores.** ~~A piece holds a flattened
  `PatternBoundary`, not the authored `SeamPath` it came from.~~ **Decided and shipped in
  code on 2026-08-17** — a piece stores its authored `SeamPath` and the polygon is derived
  at the document's tolerance, never persisted. The ADR is *still unwritten*, and that gap
  is now the sharper one: four decisions currently live only inside
  `docs/plans/2026-08-16-seampath-storage-execution-plan.md`, which is a plan, not a
  normative record.

  What it must carry when written (Task 12 of that plan):

  - **D1–D4** — the `Edge` container, `Join::Smooth` validated against coordinates rather
    than merely recorded, the bit-exact polygon→path lift, and the derived-never-stored
    boundary. Plus the C9 argument for why `SeamPath::from_boundary` does not count as
    inventing geometry while `SeamPath::closed` does.
  - **D6 — export's public signature**, answered 2026-08-17 as *project-aware*:
    `export_tiled_pdf(project, layout)`. **Both rejections must be recorded**, per the
    house rule below — a bare `tolerance_mm` parameter (rejected: it puts flattening
    policy in export's caller, which is the two-sources-of-truth failure `CutLine` exists
    to prevent), and offering both shapes (rejected: unmeasured API surface for a caller
    that does not exist). This is the only decision in the wave that changes a public
    signature in a crate outside `patal-pattern`.
  - **The v2 shape freeze itself**, once signed.

## Writing one

Keep the format the existing five use: frontmatter with `id`/`title`/`status`/`date`,
then Context, Decision, Consequences. Two things matter more than the format.

**Record what was rejected and why.** ADR-001's value is mostly in its rejected
section; ADR-003's kurbo note exists so a future reader checks a specific open issue
rather than assuming the decision was about something else.

**Record what a decision does *not* mean** when it is likely to be over-read. ADR-005
is entirely about this: unfreezing an app for development is not the same as shipping
it, and the distinction had already been lost once.
