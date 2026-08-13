# TODOS

Deferred work, with enough context that picking one up in three months does not require
re-deriving why it exists. Created 2026-08-13 by `/autoplan` during review of
`docs/plans/2026-08-13-wedge-and-validation-wave-ultraplan.md`.

Effort is given on both scales: human team, then Claude Code + gstack.

---

## P2 — Multi-piece lay plan (nesting) in export

**What.** Place several pattern pieces on one page run, instead of one piece per tile grid.

**Why.** A garment is 8-15 pieces. Printing each on its own page run wastes paper and, more
importantly, is not how a pattern is actually used: a home sewer tapes one sheet set and cuts
everything from it.

**Pros.** Turns export from a demo into something usable. **Cons.** Needs a packing algorithm,
which is a real problem (2D bin packing with rotation constraints from the grain line), not a
page transform. That is why it is not in the wave.

**Context.** `engine/crates/export/` will exist after §3.8 and owns the page transform. Nesting
sits above it and must respect `GrainLine` (added by the SeamPath blueprint §3.3), because a
piece cannot be freely rotated on napped fabric.

**Effort.** L (human) → M (CC). **Priority.** P2. **Depends on.** §3.8, and grain line landing.

---

## P2 — DXF-AAMA/ASTM emitter

**What.** The factory-facing export format, as opposed to the home-printing one.

**Why.** `docs/roadmap.md` names it alongside tiled PDF as what makes this a pattern CAD
application. Seamly2D has shipped it for years.

**Pros.** Opens the professional path. **Cons.** AAMA is a layer-and-entity convention over DXF
with real conformance surface, and this wave deliberately does not build it.

**Context.** §3.3 of the wave commits a Seamly2D reference DXF plus a written layer inventory.
Start there. **Read §5.2 first:** if that reference turned out non-conformant it was demoted
from oracle to sample, and this work is then spec-driven rather than diff-driven. ADR-008
records which of the two happened.

**Effort.** L (human) → M (CC). **Priority.** P2. **Depends on.** §3.3 reference + ADR-008.

---

## P3 — Schedule the harness's disposal

**What.** Decide when `apps/desktop` gets pruned, and write it down.

**Why.** ADR-005 calls the Tauri app disposable and non-shipping. Every wave adds a command to
it. Nothing schedules the disposal, so "disposable" is drifting toward "permanent" by default —
the exact over-reading ADR-005 was written to prevent.

**Pros.** Stops the drift while it is still cheap. **Cons.** The harness is currently the only
thing that runs on this machine, so disposal cannot precede a real UI.

**Context.** Named in §6 of the wave under "left out, now added" and explicitly not solved
there. The trigger condition is probably "when Metal canvas renders a piece on a Mac", which
means this unblocks only after Mac access exists.

**Effort.** S (human) → S (CC), and it is a decision plus an ADR, not code.
**Priority.** P3. **Depends on.** Nothing. Blocked in practice by Mac access.

---

## P3 — Grading (size runs)

**What.** Sizing a pattern up and down a size run.

**Why.** `docs/roadmap.md`: "A pattern tool that cannot grade is a drawing tool." Pure Rust,
testable without a Mac, same shape of work as export.

**Context.** Deliberately outside this wave. `PieceId` (SeamPath blueprint §3.4) exists partly
because grading indexes pieces by identity. The wave's §3.12 close-out is expected to name
grading as the nearest unbuilt pillar once export is partially built.

**Effort.** XL (human) → L (CC). **Priority.** P3. **Depends on.** v2 schema frozen.
