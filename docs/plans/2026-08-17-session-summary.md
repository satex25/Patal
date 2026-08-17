---
title: Session summary — 2026-08-17
tags: [session, summary]
updated: 2026-08-17
---

# Session summary — 2026-08-17

Executed Tasks 1–7 of the SeamPath storage wave to completion, cleared the backlog of
unmerged work, rewrote the README, and stopped deliberately at the v2 shape freeze.

Forward-looking view: [remaining-work.md](../remaining-work.md).
Current state: [status.md](../status.md).
The plan this executed: [2026-08-16-seampath-storage-execution-plan.md](2026-08-16-seampath-storage-execution-plan.md).

---

## What shipped

`main` went `4abf281` → `78ab201`. Four PRs merged, tests **136 → 168**.

| PR | What | Result |
|---|---|---|
| #5 | Tile-count overflow fix (had been sitting green, unmerged) | +2 tests |
| #6 | Incumbent persistence probe, 566 lines (same) | docs only |
| #7 | **Tasks 1–7 of the storage wave** | 163 → 168 |
| #8 | README rewritten as a Garment CAD document | docs only |

`main` is now the only branch, local and remote. Both `pre-graft-*` tags verified intact.

### The gap that closed

`apps/desktop/src-tauri/src/lib.rs` flattened a `SeamPath` and handed the polygon to
`PatternPiece::new`. The curves were gone at that line and nothing downstream could recover
them — a saved `.patal` could not be edited back into the curves it was drawn with.

A piece now stores `outline: SeamPath`. The polygon is derived on demand at the document's
tolerance and **never persisted**, so a file cannot assert an outline that disagrees with
its own geometry.

| Task | Commit | Tests |
|---|---|---|
| 1 `Edge` container | `16188a4` | 138 |
| 2 `Smooth` validated | `a60ee9f` | 144 |
| 3 the lift | `0d517ba` | 147 |
| 4 `GrainLine` | `0a9ed0f` | 154 |
| 5 `PieceId` | `9ec6a20` | 157 |
| 6 flatten tolerance | `c45d22e` | 161 |
| **7 the piece stores a path** | `c6ac313` | **168** |

Hashes changed in a rebase onto the new `main`. Anything citing `1d5e5d5`…`c071f47` points
at unreachable commits.

---

## Decisions taken

**D6 — export's public signature: option A, project-aware.**
`export_tiled_pdf(project: &Project, layout: &PageLayout)`. Rejected: a `tolerance_mm`
parameter, because it puts flattening policy in export's caller and a caller passing a
tolerance that disagrees with the document's is the two-sources-of-truth failure `CutLine`
exists to prevent; and offering both shapes, which is unmeasured API surface for a caller
that does not exist. Cost 23 call sites and a `project_of` helper per test file. Subset
export is gone until someone asks.

⚠️ **This decision currently lives only in the execution plan and `docs/adr/README.md`.
ADR-007 must carry it, with both rejections. That is Task 12 and it is non-negotiable.**

**Stop before the v2 shape freeze rather than sign it.** The freeze only pays off if Task 8
immediately follows, and Task 8 is the largest remaining task (~394 lines of spec). Signing
and stopping would spend an irreversible decision and bank nothing. Separately: the shape
had only ever been read as a six-bullet prose summary — approving a wire format nobody has
seen the bytes of is not a freeze.

**Keep "What a green build does not prove" in the README.** The PDF has never been printed
and no pattern maker has assessed the output, stated plainly beside the passing test count.

**Branch hygiene: merge, rebase, then PR.** Rebasing the wave onto the new `main` dropped
the two duplicated probe commits so the diff was purely its own work.

---

## What executing the plan proved wrong

Four defects, all corrected in the plan file rather than silently patched.

1. **The golden PDF did not change.** The plan predicted it would and supplied the re-bless
   command. Task 3's bit-exactness property makes the lifted-path flatten identical, and the
   piece list never reordered. This is *stronger* than planned: losslessness holds end to end
   through the PDF writer, not merely in the geometry tests. **A golden that moves during
   Task 7 is now a defect signal, not something to re-bless.**
2. **The offset-tightening test could not fail for the right reason.** It compared point
   counts at 0.5mm tolerance, where this curve's ~1.41× tightening lands inside a single
   adaptive-subdivision jump — both routes return 17 points, so `assert_ne!` failed against a
   *correct* implementation. Measured across allowance × tolerance; at 0.1mm they part, 45 vs
   33. The shipped test also pins the result to `flatten_for_offset` itself.
3. **`a_total_perimeter_reports_failure_…` never observed a failure.** It asserted a square's
   perimeter and stopped. Now also covers a path running out and straight back — closed,
   finite, constructible, and flattening to two distinct points.
4. **Two gate references off by one.** The freeze is before Task 8, not 9; Swift before
   Task 10, not 11.

**The generalisable lesson**, saved to memory: *a test asserting two things differ needs the
difference measured, not assumed.* Quantised algorithms — adaptive subdivision, integer bin
counts, doubling backoff — move in jumps, so a 40% parameter change can be entirely
invisible. Probe the parameter grid before hard-coding literals.

### And four stale facts in the README

Found by checking the tree rather than trusting the previous text: Node was "20+" but
`.nvmrc` pins **24.18.1**; Swift tools version **5.9** was unstated; the "correct or loud"
quote was missing a word against its source; and CI does **not** run on feature-branch
pushes — it is bound to `pull_request` and `main`. That last one matters because the macOS
jobs are the only place the Swift package is ever compiled, so *pushed is not green*.

---

## Things I got wrong in-session

Recorded because this project's discipline is that being wrong stays visible.

- Claimed the wave was **not** stacked on `incumbent-persistence-probe` after reading a
  merge-base. It was; the merge-base only showed `main` because the probe was unmerged.
  Corrected within the same turn.
- Diagnosed eight phantom `M` flags as a **CRLF** problem. Wrong — `status.md` had *more*
  CRs and was not flagged. The real cause was a stale stat cache, proven by comparing blob
  hashes (identical, `ef73990…`) before clearing anything.
- Swept `docs/scratchpad.md` into a commit with `git add -A` after having just noted it was
  never tracked. Amended out; the file is untouched.

---

## The docs folder incident

Mid-session `docs/` was renamed and ended up as `docs/docs/`, with `docs/plans/` left
behind — so the vault was split across two levels. Not committed. Sized the cost before
recommending: **23 references would have broken** across `README.md`, `CONTRIBUTING.md`,
`TODOS.md`, and two Rust source files whose doc comments cite ADR paths, plus the internal
vault links. Reverted to flat `docs/`; all 24 tracked files verified back at their original
paths with the working tree byte-identical to `origin/main`.

Worth knowing: `docs/docs/` only arose because a file had been written into `docs/plans/`
ninety seconds earlier, which recreated the parent. It was an artifact of timing, not a
layout anyone chose.

---

## State at close

**Nothing is pushed.** GitHub began returning 503s on both GraphQL and REST late in the
session, and work was deliberately kept local from that point.

| | |
|---|---|
| `origin/main` | `78ab201` — untouched since PR #8 |
| Local branch | `docs-remaining-work-checklist`, **1 commit ahead, unpushed** (`e8dcba1`) |
| Pushed pre-outage | `bce78a7` — on a branch, no PR, harmless |
| Untracked | `_to_delete/` (verified safe, kept deliberately), `docs/scratchpad.md` (never tracked) |

When GitHub recovers the branch already holds both commits, so opening the PR shows the
final state — no force-push, nothing to clean up.

Verified locally at close: **168 tests**, `fmt`, `clippy -D warnings`, `rustdoc -D warnings`,
the Tauri harness under `-D warnings`, and `cargo deny` all clean. C12 holds — Task 7 touched
zero files under `engine/crates/geometry`, and the kernel's 30 original tests are unmodified
across the whole wave.

---

## Next session, in order

1. **Chunk A of [remaining-work.md](../remaining-work.md)** — add `print_v2_shape`, run it,
   **read the actual JSON**, check it against the six points, sign or revise, delete the
   instrument. Twenty minutes, and it unblocks Task 8, grading and pattern primitives at once.
2. **Task 8** — schema v2 and the migration. Own session; it is the biggest thing left.

Two open questions worth a view before they arrive:

- **Swift: mirror or delete**, the gate before Task 10. Blueprint §6 says mirror. The counter
  is that `apps/native` is 555 lines, has no Xcode project, has never been built outside CI,
  holds no geometry, and its `Codable` is Swift-to-Swift only — a mirror nothing exercises
  drifts silently, which is the argument that deleted the Swift offset kernel. macOS access
  is expected soon, which changes the calculus.
- **What comes after the freeze — code, or validation?** Chunk C can return the answer *"the
  software is wrong"*, needs no Mac and no code, and has not moved in weeks while the engine
  moved a great deal. More engine stacked on unvalidated engine compounds risk.
