---
title: Status
tags: [status]
updated: 2026-08-13
---

# Status — 2026-08-13

Single source of truth for where the work is. Update at the end of each session;
if this disagrees with any other note, this wins.

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

## Right now

**Pushed and green.** `origin/main` is one continuous 33-commit history and local is
in sync (0/0). All five CI jobs pass: engine (ubuntu), engine (Windows), desktop,
native, and the non-blocking advisories job.

**The big unknown is resolved.** `swift build` ran against `apps/native` for the first
time in this project's history and succeeded — `Build complete! (23.10s)` — and
`swift test` executed 12 tests with 0 failures. The Swift package compiles, and it
compiles *after* the offset kernel was deleted from it, which is the version nobody
had ever built.

Rescue tags `pre-graft-backup` and `pre-graft-adr002` are kept **local only**. They
point into the pre-graft history, which is disjoint from what was pushed; publishing
them would upload a dead parallel history to the remote for no benefit. Delete them
once you are confident the graft is settled.

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

## Next, in the order I would do it

1. **Decide what a piece stores.** A `PatternPiece` holds a flattened
   `PatternBoundary`, not its authored `SeamPath` — so a saved file cannot be edited
   back into curves. This is the most likely reason for schema version 2 and should be
   settled before any file leaves this machine.
2. **Look at Seamly2D and Freesewing properly**, then write ADR-006. There is no
   competitive analysis anywhere in this project, and on the axis Pātāl currently
   competes on it is behind a free thirteen-year-old incumbent.
3. **Export.** DXF-AAMA/ASTM and tiled PDF at true scale. Pure Rust, runs on Windows,
   headlessly testable, and the cheapest route to real validation there is: print one
   at true scale and hand it to a pattern maker.

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
