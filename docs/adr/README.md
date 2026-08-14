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
- **ADR-007 — what a `PatternPiece` stores.** A piece holds a flattened
  `PatternBoundary`, not the authored `SeamPath` it came from, so a saved file cannot
  be edited back into curves. Due with the SeamPath storage blueprint; likely the
  reason for schema version 2, and it should be settled before any `.patal` file
  leaves this machine.

## Writing one

Keep the format the existing five use: frontmatter with `id`/`title`/`status`/`date`,
then Context, Decision, Consequences. Two things matter more than the format.

**Record what was rejected and why.** ADR-001's value is mostly in its rejected
section; ADR-003's kurbo note exists so a future reader checks a specific open issue
rather than assuming the decision was about something else.

**Record what a decision does *not* mean** when it is likely to be over-read. ADR-005
is entirely about this: unfreezing an app for development is not the same as shipping
it, and the distinction had already been lost once.
