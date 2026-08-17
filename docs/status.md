---
title: Status
tags: [status]
updated: 2026-08-17
---

# Status — 2026-08-17

Single source of truth for where the work is. Update at the end of each session;
if this disagrees with any other note, this wins.

**What is left, in order:** [remaining-work.md](remaining-work.md) — every unbuilt task in
one place, chunked and ordered by what unblocks what. A living page, ticked as work lands.
Start there.

**How the tree got here:**
[session summary — 2026-08-17](plans/2026-08-17-session-summary.md) — what shipped, the
decisions and their reasoning, and the four places executing the plan proved it wrong.

## Right now — 2026-08-17

**Tree.** `main` at `78ab201`. PRs #5, #6, #7 and #8 all merged this session — **eight PRs
merged total, every one green on all five CI jobs, no open PRs, and `main` is the only
branch** local and remote. `seampath-storage-wave` and `seampath-edge-container` were
deleted after merging; their commits live on in `main`. Note the wave was rebased before
merging, so anything citing `1d5e5d5`…`c071f47` points at unreachable commits.

Both `pre-graft-*` tags are intact and must stay that way — they are the only pointers into
the disjoint pre-graft history, and deleting them is the one irreversible operation in
routine cleanup.

Working tree clean apart from two untracked entries: `_to_delete/`, debris from a git
maintenance incident on 2026-08-16 that is verified safe to remove and kept deliberately
(see the checklist), and `docs/scratchpad.md`, which has never been tracked.

**Verified this cycle**, locally on Windows via `scripts\cargo.bat`, at `c6ac313`:
`fmt --check` clean · **168 tests pass** across the workspace — 167 unit and integration,
plus one doc-test · `clippy --workspace --all-targets -D warnings` clean ·
`RUSTDOCFLAGS="-D warnings" cargo doc` clean · the Tauri harness clean under `-D warnings` ·
`cargo deny check` reports advisories, bans, licenses and sources all ok.

The breakdown is spelled out because the two numbers are both defensible and the repo
had them disagreeing: `cargo test` reports 168, of which the doc-test is one. Quote the
total and say what is in it, rather than picking whichever count a given sentence needs.

**The storage wave is seven tasks in, of twelve.** `PatternPiece` now stores the `SeamPath`
the designer drew rather than the polygon it flattens to — the gap the wave exists to
close, where the Tauri harness flattened a path and handed the polygon to
`PatternPiece::new`, losing the curves at that line with nothing downstream able to recover
them. Along the way: an `Edge` container carrying per-edge joins, `Join::Smooth` validated
against the coordinates that claim it, the bit-exact polygon→path lift, `GrainLine`,
`PieceId`, and a persisted per-document flatten tolerance.

**Two results worth keeping.** The lift property — `lift(b).flatten(t)` bit-identical to
`b` at every tolerance from 1e-6 to 100 — passed first run with no epsilon and no shrunk
counterexample. And the byte-compared golden PDF did **not** move when export was rewired
through the lifted path, which the execution plan predicted it would: the losslessness
holds end to end through the PDF writer, not merely in the geometry tests that assert it.

**D6 answered: export is project-aware.** `export_tiled_pdf(project, layout)`, not
`(pieces, layout, tolerance_mm)`. A caller passing a tolerance that disagrees with the
document's is the two-sources-of-truth failure `CutLine` exists to prevent, and this is the
only shape where export cannot express a cut line the document disagrees with. Subset
export is gone until a real caller asks. **This decision is currently recorded only in the
execution plan — ADR-007 must carry it, and that is Task 12.**

**Where it stops.** At the ⛔ **v2 shape freeze**, the one-way door before Task 8. Nothing
written so far has reached a file anyone holds, which is exactly why pausing here is cheap.
See "The next decision" below.

**What landed earlier.** [The incumbent persistence probe](analysis/incumbent-persistence-probe.md)
— citation-grade evidence on the two rows that gate the v2 freeze, read from Seamly2D's
versioned XSD and source (`d6e7562`) and Freesewing's core and plugin trees (`8a8de5a`,
core v4.0.0). Two results:

- **Neither incumbent models a dart as an object.** Seamly2D persists three point
  references consumed by a `trueDarts` *tool*; Freesewing's core and all sixteen plugins
  contain `dart` zero times, while its designs contain it 194 times. Decision 2 stays
  open per the census's own instruction, but a third option now exists on the list —
  dart as a derived operation in the dependency graph — and it is the one K3 should
  discriminate against "dart as object".
- **Material belongs to the cut — both incumbents converge on the shape the census
  predicted, and one of them withdrew from it.** Freesewing's `cutlist.addCut` stores a
  *list* of `{cut, identical, onBias, onFold}` per material per part. Seamly2D shipped the
  same idea as `<mcp>` (Material, Cut number, Placement, with a role enum) across eighteen
  schema versions and **deleted it in v0.6.0**, migrating it into label prose and silently
  dropping the per-cut quantity — a lossy document migration of exactly the kind
  `patal-pattern` says must be loud. Decision 3's "we may get no evidence" escape hatch is
  no longer needed; **why Seamly2D withdrew is now the highest-value open question in this
  area**, and it is a history read rather than a drafting session.

**The probe is not K3.** No block has been drafted in either tool, no friction log exists,
and ADR-006 still must not be written from a census. Three of the four guesses it could
score were low-risk ones about whether a mature tool has features; G6 — the guess most
worth being wrong about — is barely tested.

**One thing this cycle got wrong before it got it right**, recorded because the census's
discipline is that being wrong stays visible: the probe's first draft concluded from the
current schema alone that Seamly2D "never modelled material as data". An adversarial
review pass against the sources reversed it. Reading the newest schema is not reading the
format.

## The next decision — the v2 shape freeze ⛔

**This is the live gate. Task 8 must not start without an explicit sign-off**, because it
is a one-way door: once the migration is written against a shape, changing the shape means
changing the migration.

The instrument for reviewing it is in the execution plan — a temporary `print_v2_shape`
test that serialises a two-piece project with one cubic edge, one `Smooth` join, a grain
line and a non-default tolerance, and prints it. It is a review instrument, not a
regression; delete it once the shape is signed off.

What signing off means, stated plainly:

1. A piece stores `outline` (a `SeamPath`) and **never** a polygon.
2. An edge is `{"geometry": {...}, "join": "..."}` — nested, not flat.
3. `join` may be omitted and means `corner`; `geometry` may not be omitted.
4. A piece carries `id` (bare UUID string), `grain` (nullable), `seam_allowance_mm`,
   `material`.
5. A project carries `flatten_tolerance_mm`, defaulting to 0.01.
6. **Deliberately absent:** per-edge seam allowance (P-03), fold edges (P-05), notch
   anchors (P-13). The `Edge` container is what makes each of those a field on an existing
   struct rather than a schema v3 — that is the entire argument for blueprint revision 6,
   and this is the moment it either holds or does not.

**What the freeze does not decide, and must not be read as deciding:** whether a dart is an
object. That is Decision 2 of the census, it is still blocked on K3 — hand-drafting a block
in Seamly2D and Freesewing, which is GUI work rather than code — and freezing v2 does not
settle it. If a dart turns out to be an object rather than a derived operation, it is an
*additive* piece-level field, not a change to any shape above. **Confirm that reading before
signing**, because it is the load-bearing assumption that makes signing safe while K3 is
outstanding.

Two further gates sit behind this one: **Swift mirror-or-delete** before Task 10 (blueprint
§6 recommends mirror), and ADR-007 plus the doc close-out at Task 12.

## Documentation consolidated — 2026-08-13

No code changed. The Obsidian vault was folded into `docs/` in this repository and no
longer exists as a separate location: notes are versioned, reviewable in the same diff
as the code they describe, and obtainable by anyone who clones. Wikilinks became
relative Markdown so they resolve on GitHub as well as in Obsidian. The two ultraplan
specs moved from `docs/superpowers/specs/` into `docs/plans/` alongside the session
plans, so there is one place plans live.

Two housekeeping items came with it. `reference/` — the five vendored upstream clones —
now sits at the repository root and is git-ignored; each carries its own `.git`, so
committing them would have recorded broken gitlinks. And the duplicate working copy is
gone: `C:\Users\User\Desktop\patal` is now a directory junction pointing at
`C:\Users\User\patal`, so both paths are the same repository and cannot drift.

## Where it stood — 2026-08-14

**Merged and green.** The export wave landed on `main` via PR #1
(`a9e7f1c`, 2026-08-14). All five CI jobs pass on the merge commit — engine (ubuntu),
engine (Windows), desktop, native, and the non-blocking advisories job. The
`wedge-and-validation-wave` branch was deleted locally and on origin after the merge;
`main` is the only branch.

`native` passing matters more than the other four. `swift build` needs a Mac and there
is none on this machine, so CI is the only place the Swift package is ever compiled —
it is the one job that cannot be pre-checked locally before pushing.

Local verification before the merge was done from a clean worktree checkout of the
committed tree rather than from the working directory, because those are not the same
test: engine fmt, clippy, test, doc and deny, plus the harness's clippy and tests. The
golden PDF survives checkout byte-identical, which is the failure this repo's
`.gitattributes` exists to prevent. Every commit builds on its own, so the history is
bisectable.

**A note on what "green" is worth here.** Five green jobs say the software agrees with
itself on five machines. The export wave's whole purpose is a claim no CI job can
check — see below.

**The big unknown is resolved.** `swift build` ran against `apps/native` for the first
time in this project's history and succeeded — `Build complete! (23.10s)` — and
`swift test` executed 12 tests with 0 failures. The Swift package compiles, and it
compiles *after* the offset kernel was deleted from it, which is the version nobody
had ever built.

Rescue tags `pre-graft-backup` and `pre-graft-adr002` are kept **local only**. They
point into the pre-graft history, which is disjoint from what was pushed; publishing
them would upload a dead parallel history to the remote for no benefit. Delete them
once you are confident the graft is settled.

**Branch cleanup must not sweep them up.** They are tags, not branches, and they are
the only remaining pointers into that disjoint history — a deleted merged branch costs
nothing because its commits live on in `main`, whereas deleting these makes the
history they point at unreachable and eventually collectable. It is the one
irreversible operation in routine cleanup here.

## What landed in this wave

- **Two real bugs fixed.** `offset()` reported every construction failure as
  `OffsetCollapsed`, hiding the real cause. `self_intersects()` computed the crossing
  edge indices and threw them away, so a UI could not say *where*.
- **Curves.** `SeamPath` / `EdgeSegment` / `flatten`, as a layer above the polygon
  kernel. Kernel untouched, its original tests unmodified.
- **A property suite**, which found that `serde_json` does not round-trip every f64
  without the `float_roundtrip` feature — a real defect for a CAD file format.
- **A benchmark that cancelled an optimisation.** The drag loop costs ~1% of a 120Hz
  frame at manufacturing tolerance, so the planned coarse-preview-during-drag
  strategy was dropped rather than built.
- **Material identity and a document schema version.**
- **The Swift offset kernel deleted** — 368 lines, no non-test callers, and a second
  implementation of a cutting line is a liability.
- **The Tauri app unfrozen as an engineering harness.** It is the only thing that runs
  on this machine and now draws real geometry with live sliders.
- **CI gates repaired.** Both new gates in the plan were broken as written.

## Export, on the validation track — 2026-08-13

`patal-export` exists: tiled, true-scale PDF, dependency-free writer, on the engine
side of the FFI boundary rather than in the harness. It draws `CutLine`, a newtype
with no public constructor, so no crate outside `patal-pattern` can invent a second
cut line — the rule that got the Swift kernel deleted is now a compile error rather
than a review item.

Machine-verified as far as a machine can take it. 134 engine tests green, and rendered
through pdfium the 50mm calibration square measures 50.004mm and the 200mm rule
200.008mm, stroke-centre to stroke-centre at 600 DPI.

**It has not been printed.** Every number above is the software agreeing with itself.
The claim this whole crate exists to make — that a millimetre in the model is a
millimetre on paper — is untested until a steel rule has been on it, on two printers,
and that has not happened. `docs/setup/printing.md` is the runbook for doing it.

## The primitive census — 2026-08-14

[docs/analysis/pattern-primitives.md](analysis/pattern-primitives.md) enumerates the
constructs a garment pattern needs — 34 of them — against what a `.patal` file stores.
Twenty-nine are absent. It is step K2 of the wave blueprint and it was written **before**
Seamly2D or Freesewing was opened, on purpose: a textbook supplies the vocabulary, so the
incumbents are left to answer only the question a textbook cannot, which is whether they
*persist* a construct or merely draw it. Written the other way round it would have been a
transcription of Seamly2D's feature list, and ADR-006 would be the feature table the ADR
index warns against. The commit date is the evidence of the ordering.

One finding does not need either tool and can be acted on now. Per-edge seam allowance,
fold edges, and notch positions are all attributes of an *edge*, and schema v2 as planned
already adds one — `joins`, as an array parallel to `SeamPath.segments`. Folding the others
in later means three parallel arrays that must stay the same length. Choosing the container
shape once, at v2, costs nothing today and is a breaking migration of the document's core
type later. **That decision belongs at the v2 freeze**, and the census is what makes it
visible before the door shuts rather than after.

The other two decisions the freeze actually turns on — whether a dart is an object, and
whether material belongs to the piece or to the cut — do need evidence, and drafting a
bodice block produces it as a side effect.

## Next, in the order I would do it

1. ~~**Decide what a piece stores.**~~ **Done in code, 2026-08-17.** A `PatternPiece` now
   holds its authored `SeamPath`, and the polygon is derived at the document's tolerance
   rather than stored. What remains is the **v2 shape freeze** — see "The next decision"
   above, which is the live gate.
   The [primitive census](analysis/pattern-primitives.md) narrows what the freeze has to
   get right to three decisions; the rest is additive and must not hold the door.
   **Decision 1 taken 2026-08-15** (the edge container) and now shipped. **Decision 3 has
   evidence** and is a drafting question rather than a research one — see
   [the persistence probe](analysis/incumbent-persistence-probe.md). **Decision 2 — is a
   dart an object — is the one thing still genuinely blocked on K3**, and it is therefore
   the critical path. It does not block signing the freeze, because a dart arrives as an
   additive field either way; it does block ADR-006.
2. **Look at Seamly2D and Freesewing properly**, then write ADR-006. There is no
   competitive analysis anywhere in this project, and on the axis Pātāl currently
   competes on it is behind a free thirteen-year-old incumbent. The
   [census](analysis/pattern-primitives.md) is the checklist to take in — one question per
   row, and a diff-based protocol for answering it — but ADR-006 comes from the friction
   of drafting a block in each tool, not from the census. A wedge argued from a feature
   table is what has kept this ADR unwritten through two waves.
3. **Print the thing.** Tiled PDF is built; what is missing is the half that cannot be
   automated. Print the calibration page on two printers, measure both rules and the
   square with a steel rule against the declared ±0.5mm over 200mm, record the printer
   and driver, then print a bodice block and hand it to a pattern maker. This is the
   only step that can return the answer "the software is wrong", which is why it exists.
4. **DXF-AAMA/ASTM export**, the factory-facing format. Untouched. It needs the
   Seamly2D reference capture first, and a ruling on whether that reference is an
   oracle or merely a sample. [ADR-008](adr/ADR-008-export-format-decisions.md) exists
   but deliberately does not answer that yet — it records the PDF decisions and leaves
   the DXF question open under "Still open", because there is no capture to rule on.

See [roadmap](roadmap.md) for the longer list.

## Known constraints worth not rediscovering

- **The render loop must never cross FFI per frame.** Rust hands over batched buffers;
  Metal reads them. Chatty FFI at 120Hz eats the whole frame budget and no shader
  tuning gets it back. This is a hard architectural rule and it lives in
  `docs/adr/ADR-001-stack-selection.md` in the repo, not in a scratch note.
  The measured geometry cost (~1% of a frame) says nothing about this — it is a
  statement about the boundary, not about the work on the far side of it.
- **Graphics are Metal, not `wgpu`.** An early note listed `wgpu` for "same code
  native and web". That directly contradicts ADR-001, which chose Metal for Target 1
  precisely because a portable abstraction caps the ceiling. `wgpu` is
  considered-and-rejected for Target 1; revisit only if Target 2 ever needs a shared
  renderer.
- **Flattening tolerance is not a free parameter.** Tightening it shortens chords, and
  a chord shorter than the seam allowance at a sharp corner makes the piece
  un-offsettable. See ADR-003.
- **No macOS toolchain on this machine.** `apps/native` cannot be built locally at
  all. Assume any Swift change is unverified until CI is green — CI now genuinely
  builds and tests it, so that check is real rather than theoretical.
- **Git Bash shadows MSVC's `link.exe`.** Use `scripts/cargo.bat`. rustc's own error
  message points at the wrong cause.

## Sequencing

Windows-first was and remains right: the Rust core is the bulk of the engineering and
has zero Apple dependencies. On Mac access, the SwiftUI shell and Metal go on top of a
core that is already tested and working. Nothing in the current plan is blocked on a
Mac except verifying the Swift package, which CI does.
