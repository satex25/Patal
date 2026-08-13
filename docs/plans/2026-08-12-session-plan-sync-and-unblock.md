# 2026-08-12 — Session Plan: Sync and Unblock

> **Superseded the same day by the full ultraplan blueprint:**
> `C:\Users\User\patal\docs\superpowers\specs\2026-08-12-patal-core-hardening-ultraplan.md`
> (in the code repo, so it is version-controlled). That document carries the
> researched decisions D1-D4, the task DAG, and the acceptance gates. This note
> remains useful as the plain-language survey of what was found on the day.
>
> **The blueprint was then reviewed and substantially changed** — the FFI expansion
> and the persistence crate were cut, the Swift geometry kernel was deleted rather
> than pinned with golden vectors, and two of the three proposed CI gates turned out
> to be broken as written. The review report is appended to the blueprint itself.
> [status](../status.md) carries the outcome; [roadmap](../roadmap.md) carries what was deferred.

Status: **awaiting approval**
Preceded by: [2026-08-07 rename and relocate](2026-08-07-rename-and-relocate.md)
Related: [ADR-001](../adr/ADR-001-stack-selection.md), [ADR-002](../adr/ADR-002-naming-convention.md), [the inherited-codebase analysis](../analysis/inherited-codebase.md)

---

## Part 1 — Where we actually stand (verified 2026-08-12)

Everything below was checked against the live filesystem, git, and GitHub today —
not recalled.

### The code is healthy

| Check | Result |
|---|---|
| `cargo test --workspace` | **47/47 pass** (ffi 4, geometry 25, materials 8, pattern 10) |
| Working tree | Clean, nothing untracked, `.gitignore` correct |
| Toolchain | Rust 1.97.1 pinned via `rust-toolchain.toml`, matches installed |
| Engine size | 1452 LOC across 4 crates |

The MSVC linker workaround still applies: Git Bash's coreutils `link` shadows
`link.exe`, so cargo must be invoked through a `.bat` that calls `vcvars64.bat`
first. This is a permanent property of this machine — not worth fixing in
`.cargo/config.toml` (machine-local path would break CI).

### The repo is out of sync with GitHub — and it's worse than "unpushed"

`github.com/satex25/Patal` **does exist** (private, default branch `main`, last
pushed 2026-08-03). Earlier notes said the repo had never been created — that was
wrong.

The real problem is that the local repo was created with a fresh `git init` on
2026-08-07 from an extracted zip, so **local `master` and remote `main` share no
commit history at all**. Left alone, this becomes two permanently divergent
timelines and the only ways out are a force-push (destroys 10 commits of real
history) or a merge with `--allow-unrelated-histories` (leaves a permanent
Y-shaped scar in the log).

There is a clean third option, and it works because of a lucky fact confirmed today:

```
local  8d3a447 "Import inherited Patruin codebase"  tree = f66cf4fe...
remote e71ea74 "Commit the missing gates: CI, ..."  tree = f66cf4fe...
```

**The trees are byte-identical.** The local baseline commit is an exact snapshot of
what's already on GitHub. So the 9 local rename commits can be replayed directly
onto the remote history:

```
git rebase --onto origin/main 8d3a447 master
```

This produces one continuous 19-commit history, pushes as a fast-forward, needs no
force, and loses nothing on either side.

### What is unpushed, unmerged, or unverified

- **9 rename commits** on `master` (the whole Patruin → Pātāl migration) — never pushed.
- **1 commit** on `adr-002-bundle-identifier` (`0a3ab75`, qualifies the Tauri bundle
  id to `co.satex25.patal.desktop`) — never merged, never pushed. One line, one file.
- **The Swift rename is still unverified.** `PatruinKit` → `PatalKit` was done by eye;
  no macOS toolchain exists on this machine so `swift build` has never run against it.
  CI's `native` job (`macos-latest`, real `swift build` + `swift test`) is the backstop
  — and **it only fires once we push.** This is the single strongest argument for
  pushing before doing anything else.
- **CI has run exactly once**, successfully, on 2026-08-03 — against pre-rename code.

### The two defects from the codebase audit are both still live

Confirmed by reading the source today, not inferred:

1. **Material JSON interop is broken.** `patal_materials::Material` and Swift's
   `Material` do not agree despite a doc comment claiming they mirror each other:
   - casing: `weight_gsm` / `thickness_mm` / `stretch_percent` vs `weightGSM` /
     `thicknessMM` / `stretchPercent`
   - Rust has 4 fields Swift lacks: `layer_compatibility`, `stitch_recommendations`,
     `reinforcement_requirements`, `manufacturing_considerations`
   - Swift has an extra `id: UUID` that Rust has no concept of

   Any JSON written by one side fails to decode on the other.

2. **The Swift geometry mirror is still a hand-duplicated second implementation.**
   `apps/native/Sources/PatalKit/Models/Geometry.swift` is 368 LOC re-implementing the
   offset kernel, mitre limit, winding, and self-intersection that
   `engine/crates/geometry` already does in Rust — and `patal-ffi` already exists to
   expose it. Bindings were simply never generated. Every engine change currently has
   to be made twice, and the two platforms can silently disagree on cut geometry,
   which is the one thing in a pattern CAD app that must never happen.

### The vault is thin

8 notes, ~58KB, and the structure is sound — but:

- **`Pātāl.md` (the root note) is completely empty, 0 bytes.** The vault has no
  index, no map of content, no entry point.
- **`Reminders.md` is an unstructured scratch dump** — 5 disconnected paragraphs
  mixing an architectural hard constraint (the per-frame FFI rule, which is load-
  bearing and belongs in an ADR or the architecture note) with dependency musings
  (`wgpu`, `tokio`, `bevy` — note that `wgpu` here contradicts ADR-001's Metal
  decision and needs reconciling) and a sequencing plan.
- **No status note.** Nothing in the vault tells you where the project is today
  without reading a 30KB rename plan.
- **No note covers the memorandum's roadmap** — grading, pattern primitives, the
  constraint solver, canvas, persistence, and sync are all named in
  `docs/memorandum.md` and appear nowhere in the vault.
- Nothing links to anything. There are zero ``[[wikilinks]]`` between notes, so the
  graph view is 8 isolated dots.

### Reference repos are intact

`Desktop\Pātāl\reference\` — `ferrostar`, `cargo-swift`, `uniffi-rs`, `swift-bridge`,
`XcodeBuildMCP`. Read-only, correctly outside the code repo, not dependencies.

---

## Part 2 — Plan of action

Ordered so that each step unblocks the next. Phases 1–3 are mechanical and low-risk;
Phase 4 is where the real decisions start and is deliberately left for the detailed
planning session.

### Phase 1 — Get onto GitHub cleanly *(do this first)*

Rationale for going first: pushing is what starts CI, and CI is the only thing on
Earth that can currently verify the Swift rename. Everything else is easier once
there's a remote.

1. `git remote add origin https://github.com/satex25/Patal.git`, then `git fetch origin`.
2. Rebase `master` onto `origin/main` via `git rebase --onto origin/main 8d3a447 master`.
   Verify the resulting tree is identical to pre-rebase `master` (it must be — this is
   a replay, not a merge).
3. Re-run `cargo test --workspace` on the rebased branch. Still 47/47.
4. Rebase `adr-002-bundle-identifier` onto the new `master`, then fast-forward merge it
   — it is one line and already reviewed.
5. Push. **Watch the `native` CI job specifically.** If `swift build` fails, the rename
   left something broken and fixing it becomes the immediate next task.
6. Consolidate on a single branch name. Recommend `main` (GitHub's default; CI already
   triggers on both). Local `master` gets renamed after the push lands.

**Decision needed:** `main` or `master` as the permanent default.

### Phase 2 — Repo hygiene

Small, fast, and worth doing while the remote is fresh in hand.

- Update the GitHub repo description — it currently reads "fashion app", which
  undersells a pattern CAD system and will read badly later.
- Decide whether the repo stays private. (Recommend: yes, for now.)
- Confirm the repo name casing `satex25/Patal` is what ADR-002 intends for a
  toolchain-facing surface. Recommend leaving it — a rename churns URLs for no gain.

### Phase 3 — Bring the vault to working condition

The vault should be able to answer "where is this project and what's next" in thirty
seconds. Right now it can't.

1. **Write `Pātāl.md` as a real index** — what the project is in two sentences, then
   linked entry points into Architecture, Setup, Codebase, and Plans, plus current
   status and the live open decisions.
2. **Add a status note** (`00 Status.md` or similar) — the single source of truth for
   where things stand, updated at the end of each session. This is what replaces
   "read the 30KB rename plan to find out what happened."
3. **Dissolve `Reminders.md` into where each piece belongs:**
   - the per-frame FFI constraint → promote to an ADR or the architecture note; it is a
     hard design constraint, not a reminder
   - `wgpu` / `tokio` / `serde` / `bevy` → a dependency-candidates note, **and
     reconcile `wgpu` against ADR-001's Metal decision** — right now the vault holds
     two contradicting graphics answers
   - the Windows-then-Mac sequencing → the status note
   - the App Store observation → ADR-001's context
4. **Add a roadmap note** derived from `docs/memorandum.md`, naming the unbuilt
   pillars — grading, pattern primitives, constraint solver, canvas, persistence, sync
   — so the ~95% that isn't built is visible somewhere.
5. **Wire up ``[[wikilinks]]``** across all notes so the graph is a graph.
6. Correct the crate names in `Inherited Codebase — Full Analysis.md` — it still says
   `patruin-*` throughout and predates the rename.

**Decision needed:** does the vault stay notes-only, or should it be git-tracked? It
currently isn't under version control at all, so the ADRs and audit have no history
and no backup.

### Phase 4 — The two real engineering decisions *(next session, in detail)*

These are the ones worth the "wildly in detail" planning process. Both are
architectural, both get worse the longer they wait, and both should produce an ADR
before any code moves.

**Decision A — Curves in `PatternBoundary`.**
`PatternBoundary::new(points: Vec<Point2>)` is straight-edge polygon only. Necklines,
armholes, sleeve caps, and hems are all curves — this is not an edge case, it is most
of what a garment pattern *is*. Retrofitting later means rewriting `PatternBoundary`,
`offset`, `perimeter`, `signed_area`, `winding`, `self_intersects`, and every consumer
in `pattern` and `ffi`. Roughly: polyline-with-tolerance vs. cubic Béziers vs. a
segment enum that holds both. Offsetting curves correctly is the hard part and it
drives the choice.

**Decision B — Kill the Swift mirror.**
Wire `apps/native` to `patal-ffi` through `cargo-swift` and delete the 368 duplicated
lines, or accept permanent double maintenance and silent cross-platform divergence.
The blocker is that `cargo-swift`/xcframework work needs a Mac, and Mac access is
still pending. The open question is whether there's a meaningful interim step that
can be taken on Windows — likely yes: generate and commit the uniffi bindings, and
delete the Swift geometry, so the mirror stops being maintained even before the
xcframework can be built.

Note the ordering interaction: **Decision A should land before Decision B.** Changing
the boundary representation changes the FFI surface, and generating bindings against a
type you're about to redesign means doing it twice.

The Material JSON defect is a subset of Decision B — once Swift stops hand-rolling its
own types, the mismatch disappears by construction rather than being patched. Fixing it
independently first is possible but is throwaway work if B lands soon.

---

## Open decisions summary

| # | Decision | Phase | Recommendation |
|---|---|---|---|
| 1 | `main` or `master` as default branch | 1 | `main` |
| 2 | Repo stays private? | 2 | Yes for now |
| 3 | Put the vault under git? | 3 | Yes — ADRs with no history is a real risk |
| 4 | `wgpu` vs Metal — the vault contradicts itself | 3 | Metal, per ADR-001; correct the stray note |
| 5 | Curve representation in `PatternBoundary` | 4 | Needs the detailed session |
| 6 | How and when to kill the Swift mirror | 4 | After #5 |
