# ULTRAPLAN BLUEPRINT — Pātāl Core Hardening

> Execution-ready plan produced by `/ultraplan`. Sections carry stable IDs (§1-§7)
> so the review loop can target them.

| Field | Value |
|---|---|
| **Goal (verbatim)** | "run now for all necessary processes that need to be completed with simple yet complex solutions. Create a tasklist and ensure the end goal is kept in mind... I need the code to be tip top quality with refined context included." |
| **Slug** | `patal-core-hardening` |
| **Date** | 2026-08-12 |
| **Branch** | `adr-002-bundle-identifier` @ `0a3ab75` (master @ `fd147ba`) |
| **Status** | DRAFT |
| **Execution route** | TBD |
| **Risk class** | **CUT-PATH** (Pātāl's equivalent of RISK-TOUCH: geometry that becomes a cutting line in real cloth) |

**Skill rebinding note.** `/ultraplan` is written against the SATEX trading
constitution (broker facets, 1%-per-trade risk rules, npm gates). None of that
applies here. The safety machinery is rebound to Pātāl's real equivalent, stated
in §1 under Constraints. The rule that carries over unchanged is the doctrine
already written at the top of `engine/crates/geometry/src/lib.rs`:

> "Every operation here is either correct or loud. A pattern piece that is
> silently wrong is worse than one that refuses to compute: the first gets cut
> out of cloth, the second gets fixed."

That sentence is this plan's Section 0.

---

## §1 — Objective Clarification

**Core goal.** Take the Pātāl Rust core from a well-engineered 5% prototype to a
production-quality foundation: curves that survive the cut path, invariants proven
rather than asserted, a document format settled while it is still free to change,
and an FFI surface wide enough that the home screen binds to real data instead of
stubs. Close the cross-platform divergence risk and get the repo onto GitHub so CI
becomes the backstop.

**Success criteria.** Measurable, not "looks finished":

| # | Criterion | Verified by |
|---|---|---|
| S1 | `master` on GitHub as one continuous history, no force-push | `git log` shows 19 commits, remote fast-forwards |
| S2 | CI green on all three jobs, including `native` | First `swift build` in project history passes or fails loudly |
| S3 | Curves representable and editable, offsettable through the existing kernel | Circle-oracle test passes across R x d x tolerance sweep |
| S4 | Kernel invariants proven on random input, not just 25 examples | `proptest` suite green, zero panics across all generators |
| S5 | Document format fixed before any file exists | `schema_version` present, `MaterialId` references resolve or error |
| S6 | Home screen has something to bind to | FFI exposes project/piece/material/measurement + save/load |
| S7 | Rust and Swift provably agree on cut geometry | Shared golden-vector corpus asserted by both test suites in CI |
| S8 | Vault answers "where is this and what is next" in 30 seconds | Index note, status note, `Reminders.md` dissolved, links wired |

**Constraints.** Rebound from the SATEX constitution to Pātāl's actual rules:

| # | Rule | Source |
|---|---|---|
| C1 | Correct or loud. Never return a plausible-looking number from a fallible geometry op. | `geometry/src/lib.rs` header |
| C2 | `#![forbid(unsafe_code)]` stays in every crate. | all four crates |
| C3 | The core imports no platform UI types and carries no Apple assumptions. | ADR-001 |
| C4 | The render loop never crosses FFI per frame. Rust hands over batched buffers; Metal reads them. | ADR-001, `Reminders.md` |
| C5 | `Pātāl` in prose and UI, `Patal` in anything a toolchain touches. | ADR-002 |
| C6 | `PatternBoundary` invariants live in the constructor. Private field, no back door via serde. | `geometry/src/lib.rs` |
| C7 | Wire format of `PatternBoundary` is a bare `Vec<Point2>`. Changing it breaks the Swift mirror's `Codable` contract. | `geometry` + `Geometry.swift` |
| C8 | CI gates: `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked -D warnings`, `cargo test --workspace --locked`. | `.github/workflows/ci.yml` |

**Environment.**
- Rust 1.97.1 pinned by `rust-toolchain.toml`, host `x86_64-pc-windows-msvc`.
- **Linker gotcha (permanent, machine-local):** Git Bash coreutils `link` shadows
  MSVC `link.exe`. Every cargo invocation must route through a `.bat` that calls
  `vcvars64.bat` first, invoked as `cmd //c 'path\to\script.bat'`. Do not "fix"
  this in `.cargo/config.toml`: the path is machine-local and would break CI.
- No macOS toolchain locally. `swift build` has never run against this code.
  CI `macos-latest` is the only Swift verification that exists.
- CI runners: `ubuntu-latest` (engine), `macos-latest` (desktop + native).

**Assumptions.**

| Assumption | State |
|---|---|
| 47/47 Rust tests pass | **VERIFIED** 2026-08-12 (ffi 4, geometry 25, materials 8, pattern 10) |
| Local `8d3a447` tree is byte-identical to GitHub `e71ea74` tree (`f66cf4fe...`) | **VERIFIED** 2026-08-12 |
| Rebase onto `origin/main` cannot conflict | **VERIFIED by construction**: identical base tree means every patch applies to the same content |
| `apps/desktop` (frozen Tauri) depends on `patal-geometry` + `patal-pattern` and is CI-gated with `-D warnings` | **VERIFIED** 2026-08-12 |
| Tauri's actual usage is 6 calls, none touching `material` | **VERIFIED**: `PatternBoundary::new`, `Point2::new`, `Project::new`, `PatternPiece::new`, `add_piece`, `total_perimeter_mm` |
| kurbo `CubicOffset` removed in favour of `offset_cubic`; issue #344 (endless loop, NaN from `fit_to_bezpath`) open | **VERIFIED** by research 2026-08-12; fix status in 0.12 unconfirmed |
| Swift package compiles at all | **UNVERIFIED**. No Mac. This is what S2 settles. |
| `uniffi-bindgen` runs on Windows for this crate | **UNVERIFIED**. Probed in §3.8, not depended on. |

**Unknowns.** All four resolved in the Decision Log: D1 curves, D2 file format,
D3 mirror, D4 wave scope.

---

## §2 — Domain Mapping

**Problem classification.** This is primarily a **data** and **functional** problem
with one concentrated **risk** surface. The data half is representation: what a
pattern piece *is* on disk and across the FFI boundary, settled now because file
formats become one-way doors the moment a user saves. The functional half is
capability: curves, persistence, and a boundary wide enough for a UI. The risk is
narrow and deep, confined almost entirely to `PatternBoundary::offset`, because
that function's output is a cutting line. There is no meaningful temporal or
operational dimension yet: no concurrency, no network, no background work, no
reconnect path. That keeps the blast radius small and unusually well-bounded.

**Touch-map.** SATEX agent names do not apply; these are the real subsystems.

| Subsystem | Path | Role in this wave |
|---|---|---|
| **geometry** (cut path) | `engine/crates/geometry/src/lib.rs` | 754 LOC, 25 tests. Gains `SeamPath`/`EdgeSegment`/`flatten`. Kernel itself untouched. |
| **pattern** | `engine/crates/pattern/src/lib.rs` | 343 LOC, 10 tests. `PatternPiece.material` becomes `MaterialId`; `Project` gains a library. |
| **materials** | `engine/crates/materials/src/lib.rs` | 213 LOC, 8 tests. `Material` gains identity; `MaterialLibrary` gains id lookup. |
| **ffi** | `engine/crates/ffi/src/lib.rs` | 142 LOC, 4 tests. Three functions today. Expands to the document surface. |
| **document** (new) | `engine/crates/document/` | New crate: `schema_version`, atomic save/load. |
| **native** (Swift) | `apps/native/Sources/PatalKit/` | 368-LOC geometry mirror pinned by golden vectors. `Material.swift` casing fixed. |
| **desktop** (frozen Tauri) | `apps/desktop/src-tauri/src/lib.rs` | Not a target, but a **compile gate**. Easy to forget. |
| **CI + repo** | `.github/workflows/ci.yml` | Gains `cargo-deny`, doc check, conformance check. |

**Load-bearing invariants in blast radius:**

1. `PatternBoundary` construction-only validity (>=3 finite points, deduped, private field).
2. Correct-or-loud (C1). Every new fallible op returns `Result`, never a fallback number.
3. The bare `Vec<Point2>` wire format (C7). Shared by Rust and Swift `Codable`.
4. No per-frame FFI (C4). Constrains how §3.7 shapes the render surface.
5. `forbid(unsafe_code)` (C2).

---

## §3 — Task Decomposition

Legend: ⚠️ **CUT-PATH** marks work whose output becomes a cutting line or a
persisted file. Those get the extra scrutiny in §6.

### §3.1 — Graft local history onto GitHub and push  ⛔ one-way door
- **Purpose:** start CI, which is the only thing that can verify the Swift rename.
- **Inputs:** local `master` (10 commits), `adr-002-bundle-identifier` (+1), remote `main` (10 commits).
- **Outputs:** one continuous 19-commit history on `origin/main`; three CI jobs run.
- **Depends on:** nothing. This is first.
- Subtasks:
  - Tag a rescue point: `git tag pre-graft-backup master` and `git tag pre-graft-adr002 adr-002-bundle-identifier`. Non-negotiable; costs nothing.
  - `git remote add origin https://github.com/satex25/Patal.git` then `git fetch origin`.
  - `git rebase --onto origin/main 8d3a447 master`.
  - Assert the replay changed no content: `git diff pre-graft-backup master` must be **empty**.
  - Re-run the full test suite on the rebased branch (47/47).
  - Rebase `adr-002-bundle-identifier` onto the new `master`, then fast-forward merge it (1 line, already reviewed).
  - Push. Watch the `native` job specifically.
  - Rename `master` to `main` locally and set upstream; make `main` the single default.

### §3.2 — Repo hygiene
- **Purpose:** the remote should describe the product accurately before anyone sees it.
- Subtasks: replace the "fashion app" description; confirm private status; leave the
  `satex25/Patal` name alone (a rename churns URLs for no gain, and capital-P `Patal`
  is consistent with ADR-002's ASCII form).

### §3.3 — Property-based invariants on the existing kernel  ⚠️ CUT-PATH
- **Purpose:** 25 hand-written examples prove the cases someone thought of. Random
  input proves the cases nobody thought of. This lands *before* anything near the
  kernel changes, so it is a safety net rather than a victory lap.
- **Inputs:** current kernel, unchanged.
- **Outputs:** `proptest` dev-dependency, a generator for valid boundaries, ~10 invariants.
- **Depends on:** §3.1 (so failures surface in CI).
- Subtasks:
  - Add `proptest` as a dev-dependency of `patal-geometry`.
  - Generator: random points -> convex hull -> guaranteed-valid `PatternBoundary`.
    A second generator emits arbitrary point soup, including non-finite values.
  - Invariants (convex generator): `offset(d>0).perimeter() > perimeter()`;
    `offset(d>0).signed_area().abs() > signed_area().abs()`; `offset(0)` is identity;
    winding preserved under positive offset; `offset(d)` then `offset(-d)` returns
    within tolerance of the original.
  - Invariants (all generators): every returned point is finite; a returned boundary
    never self-intersects (the function errors instead); serde round-trip is lossless;
    **the function never panics on any input**.
  - **Correctness trap to honour:** offset-then-inset is only near-identity for
    **convex** shapes. On concave input it is a morphological closing and legitimately
    loses the notch. Restricting that one invariant to the convex generator is what
    keeps the suite from being flaky-by-design.

### §3.4 — Curves: `SeamPath`, `EdgeSegment`, `flatten`  ⚠️ CUT-PATH  ⛔ one-way door (public API shape)
- **Purpose:** necklines, armholes, sleeve caps, and hems are curves. Today they can
  only be faked as polylines with no way to edit them back.
- **Inputs:** D1 decision (two-layer).
- **Outputs:** new authored types in `patal-geometry`; `PatternBoundary` **unchanged**.
- **Depends on:** §3.3 (net first).
- Design:
  ```rust
  pub enum EdgeSegment {
      Line  { to: Point2 },
      Cubic { c1: Point2, c2: Point2, to: Point2 },
  }

  pub struct SeamPath { start: Point2, segments: Vec<EdgeSegment> }  // private fields

  impl SeamPath {
      pub fn new(start: Point2, segments: Vec<EdgeSegment>) -> Result<Self, GeometryError>;
      pub fn flatten(&self, tolerance_mm: f64) -> Result<PatternBoundary, GeometryError>;
      pub fn flatten_for_offset(&self, tolerance_mm: f64, offset_mm: f64)
          -> Result<PatternBoundary, GeometryError>;
  }
  ```
- Subtasks:
  - `SeamPath::new` validates: all control points finite, path closes (auto-close with
    a `Line` if the last endpoint is not `start`), and enough segments to enclose area.
    Same doctrine as `PatternBoundary::new`: construction is the only way in.
  - Adaptive flattening by recursive subdivision on a flatness metric (control-point
    distance from the chord). Wang's bound is the reference; the circle oracle is the
    contract.
  - `flatten_for_offset` tightens tolerance because Wang's formula explicitly does not
    account for parallel-curve displacement. Scale by `1 / (1 + |offset| * k_max)` where
    `k_max` bounds segment curvature. **Treat the formula as an implementation detail
    and the oracle test as the real specification.**
  - New `GeometryError` variants: `NonFiniteControlPoint { segment }`, `PathNotClosed`,
    `ToleranceNotPositive { tolerance_mm }`. Note these are additive and the Swift
    mirror enumerates variants by hand, so §3.8's corpus must cover them.
  - Serde: `SeamPath` routes through `try_from` exactly like `PatternBoundary`, so a
    hand-edited file cannot smuggle in an unvalidated path.

### §3.5 — Document format: schema versioning + material identity  ⚠️ CUT-PATH  ⛔ one-way door
- **Purpose:** no user has ever saved a `.patal` file, so the format is free today and
  frozen forever the moment one exists. Same logic that made the bundle identifier
  free to settle on 2026-08-07.
- **Inputs:** D2 decision (settle both now).
- Subtasks:
  - Add `MaterialId` (UUID-backed) and give `Material` stable identity. This is also
    what Swift's stray `id: UUID` was reaching for.
  - `MaterialLibrary` gains `find_by_id`, `remove`, and an `add` that returns the id.
    It currently has `new`/`add`/`find_by_name`/`len`/`is_empty`/`iter` only.
  - `Project` gains `materials: MaterialLibrary` (project-owned library).
  - `PatternPiece.material` changes from `Option<Material>` to `Option<MaterialId>`.
    **This fixes a real modelling flaw:** embedding a copy means editing a library
    material leaves every piece holding a stale duplicate, which directly contradicts
    the memorandum's shareable studio libraries.
  - Unresolved id on load is an error (`MaterialNotFound { id }`), never a silent `None`.
  - `Document { schema_version: u32, project: Project }` with `schema_version = 1`.
  - Fix the Rust/Swift `Material` casing mismatch by fixing the wire contract once:
    **snake_case wins**, matching what `PatternPiece` already emits
    (`seam_allowance_mm`). Swift gets `CodingKeys`; Rust does not change.

### §3.6 — Persistence: the `.patal` document layer
- **Purpose:** the home screen needs recent projects, open, and save. None of that exists.
- **Outputs:** new crate `patal-document`.
- **Depends on:** §3.5.
- Subtasks:
  - `save(&Document, path)` writes **atomically**: temp file in the same directory,
    flush + sync, then rename. A half-written project file after a crash is
    unacceptable in a design tool.
  - Temp file is removed on **every** failure path. (This is the teardown item §6's
    CRITIC pass exists to catch.)
  - `load(path) -> Result<Document, DocumentError>` branches on `schema_version` and
    rejects versions from the future with a clear message rather than a parse error.
  - Round-trip tests, plus a corrupted-file test and a truncated-file test.

### §3.7 — FFI expansion: the surface the home screen binds to
- **Purpose:** the entire boundary today is `engine_version`, `boundary_perimeter`,
  `offset_boundary`. A home screen cannot be built on three functions.
- **Depends on:** §3.6.
- Subtasks:
  - uniffi records for `Project`, `PatternPiece`, `Material`, `Measurement`, `SeamPath`.
  - A `DocumentHandle` uniffi object owning the open document, with create / open /
    save / list-pieces / list-materials / set-measurement operations. Document ops run
    at user-action frequency, not frame frequency, so an object with interior
    mutability does not violate C4.
  - **Render surface, shaped by C4:** expose a batched vertex-buffer call
    (`piece_render_buffer(...) -> Vec<f32>`) rather than per-point accessors. Even
    though the canvas comes later, speccing the shape now is what stops a chatty
    boundary from being designed by accident. This is the single ADR-001 constraint
    most likely to be violated silently.
  - Keep `boundary_perimeter` and `offset_boundary` signatures **unchanged**: under
    D1, `PatternBoundary` never changes shape, so these stay stable forever.

### §3.8 — Swift conformance corpus (golden vectors)  ⚠️ CUT-PATH
- **Purpose:** kill the divergence risk without waiting for a Mac. The duplicate code
  stays, but it becomes impossible to merge a version that disagrees with Rust.
- **Inputs:** D3 decision.
- **Depends on:** §3.4 (so new error variants are covered).
- Subtasks:
  - A Rust test generates `conformance/vectors.json`: inputs, operation, and either
    expected output points or expected error variant. Cases drawn from the existing
    25 tests plus the new curve cases: square offset, over-inset collapse, bow-tie
    degenerate winding, acute-spike bevel, narrow-slot self-intersection, tiny-scale
    (3e-5) conditioning, extreme magnitudes (1e200, 1e-170), dedup cases, flattened
    curve offsets.
  - Corpus is committed. A CI check regenerates and diffs it, so drift cannot land.
  - `PatalKitTests` loads the same JSON and asserts identical results.
  - Both suites run in CI (engine on ubuntu, native on macos).
  - Probe whether `uniffi-bindgen` runs on Windows and record the result. Do not
    depend on it in this wave; it informs the Mac-day plan.

### §3.9 — Quality gates
- **Purpose:** "tip top quality" needs measurement, not assertion.
- Subtasks:
  - `cargo-deny` in CI (advisories, licenses, bans, sources). Already installed
    locally (0.20.2), absent from CI.
  - `criterion` benchmarks: `offset` at n = 10 / 100 / 1000, `flatten`, `self_intersects`.
  - `cargo doc --no-deps -D warnings` to catch broken intra-doc links.
  - Optional coverage report via `cargo-llvm-cov`, report-only, no gate.

### §3.10 — Vault to working condition
- **Depends on:** nothing. Fully parallel with all code work.
- Subtasks:
  - Write `Pātāl.md` as a real index. It is currently **0 bytes**.
  - Add `00 Status.md` as the single source of truth, updated at session end.
  - Dissolve `Reminders.md`: the per-frame FFI constraint is a hard architectural rule
    and belongs with ADR-001, not in a scratch file; the sequencing note moves to
    status; **the `wgpu` mention directly contradicts ADR-001's Metal decision and must
    be resolved to Metal**, with `wgpu` recorded as considered-and-rejected for Target 1.
  - Add a roadmap note from `docs/memorandum.md` naming the unbuilt pillars: grading,
    pattern primitives (darts, notches, grainlines, pleats, facings), the parametric
    constraint solver, canvas/rendering, sync.
  - Wire `[[wikilinks]]` across all notes. The graph is currently 8 isolated dots.
  - Fix `patruin-*` crate names in `Inherited Codebase — Full Analysis.md` (predates the rename).
  - **Recommendation, not yet decided:** put the vault under git. ADRs and an audit
    doc with no history and no backup is a real exposure.

### §3.11 — ADR maintenance
- ADR-001: close the stale open item. It reads "Domain of the application — not yet
  specified. Module layout and bridge choice remain blocked on it." Both are now
  settled: garment pattern CAD, and uniffi 0.28 (already in use by `patal-ffi`).
- ADR-002: crate names are listed as `patal-core`, `patal-ffi`; the real crates are
  `patal-geometry`, `patal-materials`, `patal-pattern`, `patal-ffi`.
- **ADR-003 (new):** curve representation, recording the two-layer decision, the kurbo
  #344 evidence, and the tolerance argument.
- **ADR-004 (new):** document format, schema versioning, material identity, snake_case
  wire contract.

---

## §4 — Dependency + Ordering (DAG)

**Ordered execution sequence:**
`§3.1 -> §3.3 -> §3.4 -> §3.5 -> §3.6 -> §3.7`, with
`{§3.2, §3.10, §3.11}` and `{§3.8, §3.9}` folded in as they unblock.

**Parallelizable sets:**
- `{§3.2, §3.10, §3.11}` are documentation and hygiene. No code dependency. Can run
  start to finish alongside everything else.
- `{§3.4, §3.5}` do not depend on each other. Curves touch `geometry`; the format
  touches `pattern` + `materials`.
- `§3.9` can land any time after §3.1.

**Approval nodes (one-way doors, operator sign-off required):**
- ⛔ **§3.1 push** — first publication to a shared remote, and it rewrites local branch
  history. Mitigated by the rescue tags, but it is still the point of no return.
- ⛔ **§3.4 public API** — `SeamPath` becomes the authored representation. Changing it
  later means migrating every saved file.
- ⛔ **§3.5 file format** — immutable in practice the moment a user saves.

```
§3.1 (graft+push, ⛔)
  ├──▶ §3.3 (proptest, ⚠️) ──▶ §3.4 (curves, ⚠️⛔) ──▶ §3.8 (conformance, ⚠️)
  │                                   │
  ├──▶ §3.5 (format, ⚠️⛔) ──▶ §3.6 (persistence) ──▶ §3.7 (FFI ── home screen)
  ├──▶ §3.9 (gates)
  └──▶ §3.2 (repo hygiene)

§3.10 (vault) ──┐  fully parallel, no code dependency
§3.11 (ADRs) ───┘
```

**Ordering finding that changed the plan.** The earlier working assumption was that
curves had to land before the Swift mirror could be touched, because changing the
boundary representation would change the FFI surface. Under D1's two-layer design
`PatternBoundary` never changes shape, so `boundary_perimeter` and `offset_boundary`
keep their signatures permanently. **That dependency does not exist.** §3.8 can run
in parallel with §3.4 rather than behind it.

---

## §5 — Execution Specification

### §5.1 — spec for §3.1 (graft and push)
- **Method:** replay, not merge. `git rebase --onto origin/main 8d3a447 master`.
- **Why it is safe:** `tree(8d3a447) == tree(e71ea74) == f66cf4fe...`, verified. Every
  replayed patch applies against byte-identical content, so conflicts are impossible
  by construction. This is the fact that makes a clean graft available at all.
- **Artifacts:** `origin/main` at 19 commits; rescue tags `pre-graft-backup`, `pre-graft-adr002`.
- **Validation:** `git diff pre-graft-backup master` is empty; 47/47 tests green on the
  rebased branch; all three CI jobs green.
- **Failure modes:** rebase leaves a detached or half-applied state; push rejected by a
  branch protection rule; `native` job fails because the Swift rename was wrong.
- **Fallback:** `git reset --hard pre-graft-backup` restores exactly. A `native` failure
  is not a rollback trigger; it is the finding this task exists to produce, and it
  becomes the next task.

### §5.2 — spec for §3.3 (proptest)  ⚠️ CUT-PATH
- **Method:** property-based testing. Generators produce valid boundaries (convex hull
  of random points) and hostile input (arbitrary point soup with non-finite values).
- **Artifacts:** `proptest` dev-dependency; `engine/crates/geometry/src/lib.rs` test module.
- **Validation:** `cargo test --workspace --locked` green; zero panics across all
  generators; failures shrink to a minimal reproducing case and get promoted into a
  named regression test.
- **Failure modes:** a discovered real bug in shipped code (the point of the exercise);
  a badly specified invariant producing flake, most likely the offset/inset round-trip
  applied to concave input.
- **Fallback:** if an invariant proves too strong, weaken it explicitly with a comment
  explaining why. Never delete it silently.

### §5.3 — spec for §3.4 (curves)  ⚠️ CUT-PATH
- **Method:** adaptive recursive subdivision to a flatness tolerance. Wang's formula as
  the reference bound; error converges at O(n⁶), so tolerance 0.15 -> 0.05 costs roughly
  44 -> 60 segments and the mean case needs ~3.4 subdivisions at 1e-4.
- **The tolerance argument, stated once so it is not re-derived:** industrial fabric
  cutting works to roughly 0.4mm; a flattening tolerance of 0.01mm is about 40x finer
  than any cutter can execute and far finer than cloth can hold. Flatten-then-offset is
  therefore metrologically indistinguishable from true curve offsetting **at garment
  scale**. This argument is scale-dependent and would not hold for, say, optical tooling.
- **The oracle:** a circle is the one shape whose exact offset is known in closed form
  (offsetting radius R by d gives exactly R+d). Approximate a circle with 4 cubics using
  k = 4/3·(√2−1) ≈ 0.5522847498, flatten, offset, then assert every output point lies
  within tolerance of R+d. Sweep R ∈ {10, 50, 200, 1000}mm, d ∈ {1, 5, 10, 25}mm,
  tolerance ∈ {0.1, 0.01, 0.001}. Also assert the flattened perimeter approaches 2πR
  **from below**, since chords are shorter than arcs.
- **Artifacts:** `EdgeSegment`, `SeamPath`, `flatten`, `flatten_for_offset`, three new
  `GeometryError` variants, the oracle sweep, serde round-trip tests.
- **Validation:** oracle passes across the full sweep; all 25 existing geometry tests
  still pass **unmodified** (this is the proof that the kernel was not disturbed);
  `apps/desktop` still compiles.
- **Failure modes:** tolerance too loose near tight curvature so the offset deviates
  visibly at armholes; a concave curve whose radius is smaller than the seam allowance
  produces a legitimate self-intersection.
- **Fallback:** the self-intersection case is **correct behaviour, not a bug**. The
  kernel already reports `OffsetSelfIntersects`. The product answer is for the UI to say
  "the seam allowance here exceeds the curve's radius", which the existing error surface
  already supports. Do not weaken the check to make it go away.

### §5.4 — spec for §3.5 + §3.6 (format and persistence)  ⚠️ CUT-PATH
- **Method:** identity by UUID; document envelope carrying `schema_version`; atomic
  write via temp-file-plus-rename.
- **Artifacts:** `MaterialId`, `Document`, `patal-document` crate, `MaterialNotFound`
  and `DocumentError` variants.
- **Validation:** round-trip; a corrupted file fails loudly; a truncated file fails
  loudly; a future `schema_version` is rejected with a readable message; an unresolved
  `MaterialId` errors rather than silently becoming `None`; **no temp file survives any
  failure path**.
- **Failure modes:** partial write on crash (answered by atomic rename); a
  `MaterialId` orphaned by a deleted material; `Project`'s new field breaking `Default`.
- **Fallback:** none needed for format changes while zero files exist. That freedom is
  exactly the asset this task is spending, and it does not come back.

### §5.5 — spec for §3.7 (FFI)
- **Method:** uniffi records for values, one uniffi object for the open document.
- **Validation:** FFI round-trip tests; **the C4 review question asked explicitly for
  every new function: could a UI call this per frame? If yes, batch it.**
- **Failure modes:** a chatty boundary designed by accident, which ADR-001 warns costs
  the whole frame budget at 120Hz and cannot be recovered by shader tuning.
- **Fallback:** batched buffer calls, never per-point accessors.

### §5.6 — spec for §3.8 (conformance)  ⚠️ CUT-PATH
- **Method:** golden vectors as a cross-language contract.
- **Validation:** Rust and Swift produce identical output for every vector; CI
  regenerates the corpus and diffs it so drift cannot land.
- **Failure modes:** float formatting differences between languages (compare with an
  epsilon, not string equality); error-variant naming drift between the Rust enum and
  the hand-written Swift enum.
- **Fallback:** the corpus is the contract. If they disagree, **Rust wins** and Swift
  is corrected. There is no case where the mirror is right and the engine is wrong.

---

## §6 — Risk + Ambiguity Audit (self-adversarial)

### CRITIC pass

**Assumptions not verified, and what happens if each is wrong:**

- *The Swift package compiles at all.* Never built. If the rename broke it, §3.1's CI
  run is where that surfaces. This is a known-unknown that §3.1 exists to convert into
  a known. Not a plan risk; a plan output.
- *`uniffi-bindgen` runs on Windows.* Probed in §3.8, deliberately not depended on.
- *kurbo #344 is still open.* Verified open at research time; the 0.12 fix status is
  unconfirmed. Irrelevant under D1, because kurbo is not adopted. Recorded so the
  decision does not get silently re-litigated later on stale information.

**What I left out on the first pass and had to add back:**

- **The frozen Tauri app is still a compile gate.** `apps/desktop/src-tauri` imports
  `PatternBoundary`, `Point2`, `PatternPiece`, `Project`, and CI runs
  `clippy --all-targets -D warnings` against it on `macos-latest`. "Frozen" describes
  the roadmap, not the build graph. Verified its actual usage is 6 calls and none touch
  `material`, so the real risk is **low** — but it is exactly the kind of thing that
  gets forgotten and turns into a red CI run. It is now in every validation list.
- **Temp-file cleanup on the failure path** in §3.6. The atomic-write pattern is easy to
  write and easy to leave leaking temp files on the error branch.
- **The concave offset/inset invariant** in §3.3. Specifying it over all shapes rather
  than convex ones would have produced a test suite that fails randomly and gets
  disabled, which is worse than not having written it.

**Worst case if the plan is wrong overall.** The plan spends the free-change window on
the file format. If the shape chosen in §3.5 turns out wrong after files exist, the cost
is a real migration rather than an edit. That is the considered bet, and it is why §3.5
is an approval node.

**A cost worth naming rather than hiding.** Flattening raises vertex counts from ~10 to
the hundreds. `self_intersects()` is O(n²) and runs inside every `offset()` call, which
runs inside `cut_boundary()`. At n=800 that is roughly 640k cross products: fine at
edit frequency, **not** fine inside a 120Hz frame budget. The answer is architectural,
not algorithmic: cut boundaries get cached and recomputed on edit, never per frame,
which is the same discipline C4 already imposes. §3.9's benchmarks exist to measure
this before it bites rather than after. Do not pre-optimise to a sweep-line; measure first.

### RISK-AGENT pass (rebound to Pātāl's rules)

| Rule | Check | Verdict |
|---|---|---|
| C1 correct-or-loud | Every new fallible op returns `Result`. Flatten tolerance is explicit, never silently defaulted. Unresolved `MaterialId` errors. | PASS |
| C2 no unsafe | No task introduces `unsafe`. | PASS |
| C3 core purity | New crates are `document` (std fs only) and geometry types. No UI types. | PASS |
| C4 no per-frame FFI | §5.5 makes this an explicit per-function review question, and §3.7 specs a batched render buffer. | PASS |
| C6 constructor invariants | `SeamPath` routes serde through `try_from`, matching `PatternBoundary`. | PASS |
| C7 wire format | `PatternBoundary` stays a bare `Vec<Point2>`. `SeamPath` is additive. | PASS |
| Cut-path integrity | The 754-LOC kernel is not modified. All 25 tests must pass **unmodified**. | PASS |

**Verdict: APPROVED.**

One revision was forced during this pass. An earlier draft had §3.4 modifying
`PatternBoundary` directly to hold segments. That would have put curve-curve
intersection on the cut path and required rewriting `self_intersects`, `signed_area`,
`winding`, and `offset` at once. It was **VETOED** against the cut-path integrity rule
and replaced by the two-layer design, which leaves the kernel untouched. D1 confirmed
that direction.

**Unresolved items the operator owns:**
- Whether the vault goes under git (§3.10). Recommended yes. Not blocking.
- Mac access timing, which gates the xcframework and the mirror's actual deletion.
  §3.8 is specifically designed so this does not block anything.

---

## §7 — Final Assembly

**Build order.**

1. Tag rescue points, add remote, fetch. -> two tags -> `git tag -l` shows both.
2. Rebase `master` onto `origin/main`. -> 19-commit history -> `git diff pre-graft-backup master` empty.
3. Re-run tests, merge ADR-002, push, watch CI. -> green pipeline -> `native` job reports for the first time.
4. Consolidate on `main`. -> one default branch.
5. Repo description + vault index/status notes. -> §3.2, §3.10 -> vault answers "where are we" in 30s.
6. proptest harness. -> ~10 invariants -> zero panics on random input.
7. `SeamPath` + `flatten` + oracle. -> curves -> oracle sweep green, **25 kernel tests unmodified**.
8. `MaterialId`, `Document`, `schema_version`. -> settled format -> unresolved id errors loudly.
9. `patal-document` save/load. -> atomic persistence -> corrupted/truncated files fail loudly, no temp leaks.
10. FFI expansion. -> home-screen surface -> every new call passes the C4 question.
11. Golden-vector corpus. -> cross-language contract -> Rust and Swift agree in CI.
12. `cargo-deny`, benchmarks, doc check. -> quality gates -> CI enforces all four.
13. ADR-001 close-out, ADR-003, ADR-004. -> decisions recorded -> no stale open items.

**Acceptance criteria (gate outcomes).** The SATEX npm gates do not exist here; these
are the real ones:

- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings` clean
- [ ] `cargo test --workspace --locked` green, count reported and **>= 47**
- [ ] `cargo deny check` clean (new gate)
- [ ] `cargo doc --no-deps` clean (new gate)
- [ ] `apps/desktop` (frozen Tauri) still compiles under `-D warnings`
- [ ] CI `native` job runs `swift build` + `swift test` and reports
- [ ] All 25 pre-existing geometry tests pass **unmodified**
- [ ] Circle oracle passes the full R x d x tolerance sweep
- [ ] proptest reports zero panics across all generators
- [ ] Golden-vector corpus asserted identical by both languages
- [ ] No temp file survives any persistence failure path

**Deliverables.** This blueprint; ADR-003 and ADR-004; vault index, status, roadmap, and
dissolved `Reminders.md`; `patal-document` crate; `conformance/vectors.json`; a CI
workflow with four gates instead of three.

---

## Decision Log

| D# | Question | Chosen | Why |
|---|---|---|---|
| D1 | Curve representation | **Two-layer: `SeamPath` + `flatten`** | Keeps the 754-LOC kernel and its 25 tests untouched; avoids kurbo's open endless-loop/NaN bug on the cut path; matches industry practice (parametric authoring, discretized manufacturing); at 0.01mm the error is ~40x finer than any cutter. |
| D2 | File format and material identity | **Settle both now** | Zero files exist, so the format is free today and frozen forever after the first save. Also fixes the stale-copy flaw where an edited library material leaves pieces holding duplicates. |
| D3 | Swift mirror before Mac access | **Golden-vector conformance** | Kills the actual danger (silent divergence) without a Mac, and the same corpus later validates the real bindings. |
| D4 | Wave scope | **Core + document layer + FFI** | The FFI is three functions today; a home screen has nothing to bind to without this. Excludes the constraint solver, which is a project in its own right. |

*(The skill's D1 boundary-confirmation gate was waived on explicit operator
instruction to run without stopping. The boundary was stated before research began.)*

## Revision Log

| # | Section | Change | Trigger |
|---|---|---|---|
| 1 | §3.4 | Segment-enum-in-`PatternBoundary` replaced by two-layer design | §6 RISK-AGENT veto on cut-path integrity, confirmed by D1 |
| 2 | §4 | §3.8 moved from "blocked by curves" to parallel | `PatternBoundary` shape is stable under D1, so FFI signatures never change |
| 3 | §6 | Added Tauri compile-gate risk | CRITIC pass; verified `apps/desktop` imports domain crates and is CI-gated |

---

# GSTACK REVIEW REPORT (`/autoplan`, 2026-08-12)

| Phase | Voice | Status |
|---|---|---|
| Preflight | Codex CLI | **UNAVAILABLE** (binary not installed). All phases degrade to `[subagent-only]`. |
| 1. CEO | Claude subagent | **COMPLETE**. 14 findings, 4 critical. |
| 2. Design | n/a | **SKIPPED**. No UI scope: the plan explicitly excludes UI design. |
| 3. Eng | Claude subagent | **INCOMPLETE**. First run exceeded the output token cap; relaunch was interrupted. **Owed.** |
| 3.5 DX | Claude subagent | **COMPLETE**. 26 findings, 3 critical, several verified by execution. |

Consensus tables are not meaningful with one voice per phase. Every finding below is
single-source and was re-verified against the repo before being recorded, or is
explicitly marked unverified.

## Facts established by direct verification (not reported, checked)

| Claim | Verdict |
|---|---|
| No Xcode project or workspace exists anywhere in the repo | **TRUE**, `find` returns nothing |
| `ContentView.swift` is a 47-line placeholder whose only action appends an empty in-memory `Project` | **TRUE** |
| `cutBoundary()`'s only caller is `PatalKitTests.swift:158`; the 368-line Swift offset kernel is unreachable from any product code | **TRUE** |
| Tauri runs on Windows today, calling `engine_demo_perimeter_mm` via `invoke`, linking the Rust crates directly with no FFI | **TRUE** |
| `apps/desktop` is a CI-gated compile target despite being "frozen" | **TRUE** (recorded earlier in §6) |

## Findings accepted, with the plan changes they force

**A1 (critical). §3.7's FFI expansion has no reachable consumer.** It was justified by
"the home screen needs something to bind to." There is no Xcode project, no Mac, no
generated bindings, and §1 already lists `uniffi-bindgen` on Windows as UNVERIFIED.
The surface would be exercised by Rust-side round-trip tests only, for months.
**Change: cut §3.7 from this wave.**

**A2 (critical). The one runnable platform is the frozen one.** ADR-001 rejected Tauri
as a *shipping* target on native-feel grounds. The plan silently extended that to the
*development* platform and demoted the only thing that runs on this hardware to a lint
target. **Change: unfreeze Tauri as an explicitly non-shipping, disposable engineering
harness.** Curves get a visual consumer, `.patal` gets exercised by a save actually
performed, and none of it touches ADR-001's shipping decision. Record as ADR-005 so it
cannot be misread as reversing ADR-001.

**A3 (critical). D3 was chosen on bad information, and the choice should be revisited.**
The golden-vector corpus (§3.8) builds permanent CI machinery to pin code that has zero
non-test callers and a scheduled death date. When D3 was put to the operator, option C
("delete the Swift geometry") was described as leaving the package "visibly degraded."
That description was **wrong**: nothing degrades, because nothing depends on it.
**Change: surface this at the gate as a User Challenge.** Deleting the offset/winding/
self-intersection code from `Geometry.swift` drops divergence risk to zero permanently
in an afternoon, versus taxing every future `GeometryError` change in perpetuity.

**A4 (critical, verified by execution). Two of the three new CI gates in §3.9 are broken
as written.**
- `cargo deny check` fails on the current tree with roughly 40 errors: 34 license
  rejections, 4 workspace crates flagged `unlicensed` (they are `publish = false` with
  no `license` field), plus live advisories RUSTSEC-2024-0436 (`paste`, unmaintained,
  reached through **uniffi 0.28**, the dependency §3.7 wanted to expand) and an
  unmaintained `bincode`. As specified the gate goes red on the merge commit and stays
  red. **Change: a `deny.toml` with an explicit allow list, `private.ignore = true`, and
  documented advisory ignores is a prerequisite, not a follow-up. Run `advisories` as a
  scheduled non-blocking job so an overnight RUSTSEC publication cannot redden an
  unrelated PR.**
- `cargo doc --no-deps -D warnings` is **not a valid command**; `cargo doc` rejects `-D`.
  The working form is `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`. The
  acceptance checklist then contradicts §3.9 by dropping `-D warnings` entirely.

**A5 (high). §3.8's corpus check contradicts itself and will flake across runners.**
§3.8 says CI "regenerates and diffs" (byte equality) while §5.6 says floats must be
compared with an epsilon. Corpus generation happens on Windows/MSVC and the diff on
`ubuntu-latest`/glibc; `hypot` and `sqrt` are not bit-identical across libm
implementations, so one ULP reddens CI with a diff no contributor can act on.
**Change: parse both corpora and compare numerically. Never `git diff --exit-code` the
file. Pin generation to one runner OS.**

**A6 (high). The corpus would create the very duplication it exists to remove.**
`GeometryError` does not derive `Serialize`, and Rust variants are PascalCase while
Swift's are camelCase. Encoding "expected error variant" therefore needs a new
hand-written Rust-to-string map plus a matching Swift one: a second hand-maintained
cross-language table. **Change: derive `Serialize` on `GeometryError` with
`#[serde(tag = "kind", rename_all = "snake_case")]` so the encoding is generated.**

**A7 (high, real bug in shipped code). `geometry/src/lib.rs:376-377` discards a real
error and reports a misleading one.** `PatternBoundary::new(new_points).map_err(|_|
GeometryError::OffsetCollapsed { distance_mm })?` swallows the inner `GeometryError`.
A `NonFiniteCoordinate` or `TooFewPoints` is reported to the caller as "collapsed the
boundary through itself." This is the single place in the crate that reports a
plausible-looking wrong error, which is precisely what the file header forbids.
**Change: propagate the inner error or add `OffsetProducedInvalidBoundary { source }`.
This is a defect in existing code, independent of the plan.**

**A8 (high). §5.3 promises a UI message the error surface cannot produce.** The plan
says the UI can tell the designer "the seam allowance **here** exceeds the curve's
radius" and that "the existing error surface already supports" it. It does not.
`OffsetSelfIntersects { distance_mm }` carries only the distance, and
`self_intersects()` returns a bare `bool`, computing the crossing edge indices and
throwing them away. **Change: return `Option<(usize, usize)>` and carry the indices in
the variant, while §3.4 is already extending the enum.**

**A9 (high). The O(n²) mitigation does not cover the dominant interaction.** §6 answers
the `self_intersects` cost with "cache cut boundaries, recompute on edit, never per
frame." Dragging a control point *is* a stream of edits at frame rate, so "on edit" and
"per frame" are the same thing during the interaction that matters most, and it drags a
full FFI round trip with it (violating C4). **Change: benchmark the real path
(`flatten → offset → self_intersects` on a 4-cubic neckline) against an 8.3ms budget,
and decide the coarse-preview-flatten-during-drag strategy before `SeamPath`'s shape is
fixed.**

**A10 (high). The file-format one-way-door argument is weaker than stated.** A bundle
identifier is immutable by Apple's external rule. A file format is immutable only once
files exist in someone else's hands, and the only holder of every `.patal` file for the
foreseeable future is the person who would write the migration. Worse, the schema is
being frozen while grading, darts, notches, grainlines, and the constraint solver are
all explicitly unbuilt, which makes `schema_version = 2` close to certain.
**Change: keep `schema_version` and `MaterialId` (both are cheap and independently
correct; the stale-copy argument stands on its own merits). Drop the "settle the format
forever" framing and its ⛔ approval gate.**

**A11 (high). No competitive analysis exists anywhere.** The plan, the memorandum, and
both ADRs contain zero mentions of any competitor. Valentina/Seamly2D is free, open
source, parametric, roughly 13 years old, and ships DXF-AAMA/ASTM export and tiled PDF
printing. On the axis Pātāl currently competes on (draw a polygon, offset a seam
allowance) it is behind a free incumbent. **Change: ADR-006 stating the wedge, after
actually drafting one bodice block in Seamly2D and Freesewing.**

**A12 (high). The two capabilities that define a pattern CAD app are absent from the
plan.** No export (DXF-AAMA/ASTM, tiled PDF at true scale) and no grading. Both are pure
Rust, both run on Windows with no Mac, both are headlessly testable, and export is the
cheapest possible route to real validation: print a tiled PDF and hand it to a pattern
maker. **Change: strong candidate to replace §3.6 + §3.7 in this wave.**

**A13 (critical for onboarding, verified). The only working build path is three broken
`.bat` files in `%TEMP%`.** `patal-cargo-{test,fmt,clippy}.bat` point at
`.worktrees/patal-rename/engine`, which no longer exists; running one now errors. They
live in a directory Windows deletes on its own schedule. The Git Bash `link.exe`
shadowing is documented **only inside this plan file**, which is itself untracked, and
rustc's own error tells you to repair Visual Studio, which does not help.
**Change: commit `scripts/cargo.bat` using `%~dp0` so it is path-independent, plus a
README Prerequisites paragraph naming the `/usr/bin/link: extra operand` symptom.**

**A14 (high, verified). README states something factually false.** It claims "nothing in
`engine/` derives `Serialize`/`Deserialize` yet, so no document can currently leave
process memory." Every domain type derives both, with passing JSON round-trip tests.
A returning author would conclude the serde work is unstarted when it is essentially
done, which is exactly the misread that makes §3.6 look larger than it is.

**A15 (high). CI never tests Windows**, the only platform the author develops on.
Adding a `windows-latest` job running `cargo test --workspace --locked` is the cheapest
job in the matrix and covers 100% of local development.

**A16 (medium). ADRs are cited normatively by README and the memorandum but do not
exist in the repo** (they live in the Obsidian vault). A fresh cloner cannot obtain
them, and §3.11 would write ADR-003 and ADR-004 into the vault, deepening the problem.
**Change: move ADRs to `docs/adr/`. They are engineering artifacts and belong beside
the code they constrain.** This also resolves the earlier open question about putting
the vault under git, for the documents that actually need it.

**A17 (medium). §3.3's framing is wrong.** proptest is described as a safety net landing
before kernel changes, but the plan states in three places that the kernel is not
modified. There is nothing for the net to catch. Its real value is latent-defect
discovery in shipped code, which is genuine but not blocking.
**Change: reframe honestly and demote from "must land first" to "runs in parallel."**

**A18 (medium). No time estimates anywhere**, in a plan whose only scarce resource is
solo-developer weeks. Thirteen build steps, three gates, twelve acceptance criteria,
zero durations. **Change: day estimate per section before executing anything.**

**A19 (medium, two internal inconsistencies).**
- S1 says the push produces 19 commits. Remote has 10 and local replays 9, but §7 step 3
  merges ADR-002 *before* pushing, making it **20**. The criterion fails against the
  plan's own build order.
- §3.2 justifies keeping `satex25/Patal` as "consistent with ADR-002's ASCII form."
  ADR-002's table specifies `Git repository: patal`, lowercase, and README links to the
  lowercase URL. The decision to leave it alone is fine; the stated justification
  contradicts the ADR it cites.

**A20 (medium). `#[uniffi(flat_error)]` may be the wrong long-term call.** It flattens
every error to a string, so a consumer cannot distinguish `OffsetSelfIntersects` (user
error, offer to reduce the allowance) from `NonFiniteCoordinate` (engine bug), and the
English string becomes the API. Changing it after §3.4 adds three variants is a breaking
change for every consumer. Deferred with §3.7 but should be decided when FFI returns.

## Findings rejected or corrected

**R1. The tolerance-scaling formula is NOT wrong in the inset direction.** The CEO voice
claimed `1/(1 + |d|·k_max)` under-tightens on insets by an unbounded factor and that the
correct inward factor is `(1 − dk)`. Working it through: flattening sagitta error is
proportional to local radius, and an offset by signed distance `d` along the outward
normal maps radius `R` to `R + d`. For a convex region (`R > 0`), outward offset
amplifies by `(R+d)/R = 1 + dκ` and inward offset *shrinks* error. For a concave region
(`R < 0`), the signs invert: outward shrinks, inward amplifies by `1 + |d||κ|`. Because
the sign of `d` flips *which regions* amplify, the worst-case amplification is
`1 + |d|·|κ_max|` in **both** directions. The plan's formula is therefore correct and
conservative. The unbounded case the CEO voice pointed at is the offset self-intersection
that the kernel already detects, not a tolerance error.
**This correction is unverified by a second engineering voice and should be confirmed
when the Eng review is completed.**

**R2. The valuable half of that same finding stands.** The oracle sweep uses only
positive `d` **and only circles, which are entirely convex**. It therefore never tests
an inset and never tests a concave region: exactly the two cases where the amplification
analysis above is non-trivial. **Change: extend the sweep to negative `d` and to a shape
with concave curvature, asserting either the analytic offset radius or a loud error when
`|d|·κ ≥ 1`.**

## Still owed before this review is complete

1. **The Eng review.** Not delivered. It specifically owed independent confirmation of
   R1's algebra, an architecture assessment of the two-layer design, and edge-case
   analysis. R1 currently rests on one derivation with no second opinion.
2. **Consensus tables.** Not meaningful at one voice per phase; would need Codex
   installed (`codex login`) to produce real ones.
3. **Time estimates** (A18), which the CEO voice argued are the estimate that decides
   whether the back half of the wave is defensible at all.

## Net effect on the wave

Surviving roughly intact: §3.1 graft and push, §3.3 proptest (reframed, demoted),
§3.4 curves (with the oracle gap in R2 closed, the node-continuity question answered,
and the ⛔ label reconsidered), §3.10 vault, §3.11 ADRs (relocated to `docs/adr/`).

Cut or deferred: §3.7 FFI (A1), §3.6 persistence (A1/A12 knock-on), §3.8 corpus in its
current form (A3/A5/A6).

Added: Tauri harness (A2), export and grading as the candidate replacement wave (A12),
`deny.toml` and the corrected doc gate as prerequisites (A4), `scripts/cargo.bat` plus
README prerequisites (A13), Windows CI job (A15), the `OffsetCollapsed` bug fix (A7),
crossing-index plumbing (A8), the drag-loop benchmark (A9).

## Decision Audit Trail

| # | Phase | Decision | Class | Principle | Rationale |
|---|---|---|---|---|---|
| 1 | 0 | Skip Design phase | Mechanical | P3 | Plan excludes UI design; a design review would have nothing to evaluate |
| 2 | 0 | Run DX phase | Mechanical | P1 | Library plus FFI surface plus a notorious build gotcha is squarely DX scope |
| 3 | 0 | Do not upgrade gstack mid-review | Mechanical | P6 | Skill files would change under a running review |
| 4 | 0 | Do not auto-commit despite `checkpoint_mode: continuous` | Mechanical | P6 | Task #1 is a history graft; stray commits complicate the rebase |
| 5 | 0 | Do not add CLAUDE.md routing rules | Mechanical | P6 | Same reason: an unrelated commit immediately before a graft |
| 6 | 1/3.5 | Run the two independent voices in parallel | Mechanical | P3 | Both are specified context-free; only Codex consumed prior-phase output, and it is unavailable |
| 7 | 1 | Accept A1, cut §3.7 | Taste | P3 | Verified there is no consumer; reasonable people could still argue for designing the surface early |
| 8 | 1 | Accept A2, unfreeze Tauri as a harness | Taste | P1+P2 | Highest-leverage change in the review, but it does touch a frozen decision |
| 9 | 1 | Accept A10, drop the format freeze framing | Mechanical | P5 | The bundle-id analogy does not hold; keep the cheap parts |
| 10 | 3.5 | Accept A4, treat `deny.toml` as a prerequisite | Mechanical | P1 | Verified failing; a red-on-arrival gate is worse than no gate |
| 11 | all | Reject R1's math, keep R2's test gap | Mechanical | P1 | Derivation shows the formula is conservative in both directions |
| 12 | 1 | A3 (delete the Swift kernel) escalated, not auto-decided | **User Challenge** | n/a | Overturns operator decision D3, which was made on a description now known to be wrong |

---

# EXECUTION RECORD (2026-08-12)

Built on branch `core-hardening-wave-1`, on top of the verified graft. Every claim
below was run, not reasoned about.

| Gate | Result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --workspace --all-targets --locked -D warnings` | clean |
| `cargo test --workspace --locked` | 89 pass (was 47) |
| `cargo deny check` | clean, all four checks (was ~40 errors) |
| `RUSTDOCFLAGS=-D warnings cargo doc --no-deps --workspace` | clean |
| `apps/desktop` under `-D warnings` | clean, plus 4 tests |
| Original 25 geometry tests unmodified | yes |
| Circle oracle across R x d x tolerance | passes, both signs of d |
| proptest, 1,000,000 cases | zero failures |
| `apps/native` | **unverified — no macOS toolchain. CI is the only build.** |

## Operator decisions taken during execution

| # | Question | Chosen |
|---|---|---|
| D3 (re-decided) | Swift geometry mirror | **Delete it.** The original choice rested on a description — "visibly degraded" — that was verified false. |
| — | How far to take §3.1 | Graft and verify locally; **stop before `git push`**. |
| — | Vault under git | No. ADRs move to `docs/adr/`; the vault stays loose. |

## What the plan said, and what execution found

**§3.4's oracle could not have worked as specified.** It called for approximating the
circle with four cubics. A cubic cannot represent a circular arc exactly — a
quarter-arc is off by 2.7e-4 of the radius, which on a 1000mm circle is 0.27mm against
the 0.001mm tolerance the sweep is meant to verify. As written it measures how badly
cubics approximate circles. Now 32 arcs, with the quarter-arc figure pinned in its own
test so the error budget is explicit.

**§5.3's tolerance formula is conservative, and now the tight bound is known.** The
worked derivation gives `max(1, |d|·κ)`, not `1 + |d|·κ`. The plan's formula is
therefore correct and over-tightens by up to 2x. Kept anyway, deliberately, for
reasons recorded in ADR-003. This closes the second opinion the incomplete Eng review
owed; it is empirical rather than a second human voice, which for this particular
claim is the stronger of the two.

**Nothing in the plan anticipated that tolerance and seam allowance interact.**
A corner of turn `θ` consumes `d·tan(θ/2)` from each adjacent edge. Tighten the
tolerance far enough and a chord next to a sharp corner becomes shorter than the
allowance, that edge reverses, and the kernel correctly refuses. Same shape, same
allowance: succeeds at 0.01mm, fails at 0.001mm. Pinned as a test, recorded in ADR-003
as unsolved.

**§3.3's proptest suite found two real defects**, which is what it was reframed to be
for. `serde_json` does not round-trip every f64 without the `float_roundtrip` feature
— a genuine problem for a CAD file format. And `offset(0.0)` returns its input
unchanged, self-crossing or not, which is correct but means the no-crossing guarantee
belongs only to offsets that actually constructed something.

**§6's O(n²) worry does not survive measurement.** The full drag path costs about 1%
of a 120Hz frame at manufacturing tolerance, and 7.7% at forty times finer than any
cutter can execute. The coarse-preview-during-drag strategy was **dropped rather than
built**.

**A19 confirmed:** the push is 20 commits, not 19.

## Cut, and still cut

§3.7 FFI expansion (no reachable consumer), §3.6 persistence crate (knock-on), §3.8
golden-vector corpus (obsoleted by deleting the code it would have pinned).

## Owed, in priority order

1. **The push.** Held at the operator gate. `main` is 10 ahead / 0 behind and
   fast-forwards; rescue tags exist. Until it happens CI has still never compiled the
   Swift package — and that package changed substantially in this wave.
2. **What a piece stores.** `PatternPiece` holds a flattened `PatternBoundary`, not
   its authored `SeamPath`. A file written today cannot be edited back into curves.
   This is the most likely reason for schema version 2 and should be settled before
   any file leaves this machine. Discovered while writing ADR-004, not while planning.
3. **Node continuity** between adjacent segments. A designer dragging a handle across
   a smooth join breaks tangency with nothing to stop them. Needed before a curve
   editor.
4. **A18 — time estimates.** Still absent. The CEO voice argued these are what decide
   whether the back half of a wave is defensible at all, and that argument was never
   answered. It is now answerable with evidence rather than guesswork: this wave was
   one working session.

## The two strategic gaps (A11, A12), recorded rather than acted on

Both are scoping decisions rather than engineering ones, so neither was taken
unilaterally.

**No competitive analysis exists anywhere.** Seamly2D/Valentina is free, open source,
parametric, roughly thirteen years old, and already ships DXF-AAMA/ASTM export and
tiled PDF printing. Freesewing is parametric-by-code with a real user base. On the axis
Pātāl currently competes on, it is behind a free incumbent. `docs/adr/README.md` names
ADR-006 as where this goes, and says it should be written after drafting one bodice
block in each rather than from a feature table.

**Export and grading are absent from every plan**, and they are the two capabilities
that make this a pattern CAD application rather than a drawing program with a garment
theme. Both are pure Rust, both run on Windows with no Mac, both are headlessly
testable. Export is also the cheapest route to real validation that exists: print a
tiled PDF at true scale and hand it to a pattern maker. Recorded in the root README
and in the vault's Roadmap note.

These two are the strongest candidates for the next wave, ahead of resuming the FFI.
