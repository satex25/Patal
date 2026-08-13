# ULTRAPLAN BLUEPRINT — What a Pattern Piece Stores

> Execution-ready plan produced by `/ultraplan`. Sections carry stable IDs (§1-§7)
> so the review loop can target them.

| Field | Value |
|---|---|
| **Goal (verbatim)** | "Ultraplan the SeamPath-storage gap before touching export/grading" |
| **Slug** | `patal-seampath-storage` |
| **Date** | 2026-08-13 |
| **Branch** | `main` @ `dc509eb` (clean, up to date with `origin/main`) |
| **Status** | DRAFT |
| **Execution route** | TBD |
| **Risk class** | **CUT-PATH** — `cut_boundary()` output becomes a cutting line |

**Skill rebinding note.** `/ultraplan` binds to the SATEX trading constitution. None
of it applies here. The safety machinery is rebound to Pātāl's real rules, carried
forward unchanged from the core-hardening blueprint (`2026-08-12-...-ultraplan.md`,
§1 Constraints) and restated in §1. Section 0 remains the doctrine at the head of
`engine/crates/geometry/src/lib.rs`:

> "Every operation here is either correct or loud. A pattern piece that is silently
> wrong is worse than one that refuses to compute: the first gets cut out of cloth,
> the second gets fixed."

---

## §1 — Objective Clarification

**Core goal.** Make the authored curve the thing a `.patal` file stores, so a saved
piece can be edited back into curves instead of arriving as a frozen polygon. Settle
the remaining storage questions in the same schema bump, and build the migration
mechanism once, while the only file that exists is disposable.

**The gap, demonstrated rather than described.** `apps/desktop/src-tauri/src/lib.rs:142-148`:

```rust
let path = bodice_front()?;                                  // SeamPath: 5 segments, 2 cubic
let boundary = path.flatten(tolerance_mm)?;                  // → polygon
let mut piece = PatternPiece::new("Bodice Front", boundary); // SeamPath dropped here
```

That is the document `save_demo_document` writes. The curves are gone at line 148 and
nothing downstream can recover them. `PatternPiece` is declared at
`engine/crates/pattern/src/lib.rs:106-120` with `pub boundary: PatternBoundary` and no
authored representation at all.

**Success criteria.**

| # | Criterion | Verified by |
|---|---|---|
| S1 | A piece stores what the designer drew | `PatternPiece` holds `SeamPath`; no `PatternBoundary` field survives |
| S2 | Round-tripping a file returns editable curves | Harness reloads and reports segment count and cubic count, not just vertices |
| S3 | The polygon is derived, never persisted | No `PatternBoundary` appears anywhere in a serialized `.patal` |
| S4 | Lifting a polygon into a path loses nothing | proptest: `lift(b).flatten(t)` is bit-identical to `b` for every `b` and every `t` |
| S5 | A file determines its own cut line | Tolerance persisted; save → load → `cut_boundary` yields identical points |
| S6 | v1 files migrate rather than being refused | Committed v1 fixture loads, migrates, and its outline flattens bit-identical to the v1 boundary |
| S7 | A smooth join cannot lie | `Join::Smooth` with non-collinear handles is rejected at construction |
| S8 | The kernel is still untouched | All 31 pre-existing `geometry/src/lib.rs` tests pass unmodified |

**Constraints.** Carried from the core-hardening blueprint, same numbering.

| # | Rule | Source |
|---|---|---|
| C1 | Correct or loud. Never return a plausible-looking number from a fallible op. | `geometry/src/lib.rs` header |
| C2 | `#![forbid(unsafe_code)]` stays in every crate. | all four crates |
| C3 | The core imports no platform UI types. | ADR-001 |
| C4 | The render loop never crosses FFI per frame. | ADR-001 |
| C5 | `Pātāl` in prose and UI, `Patal` in anything a toolchain touches. | ADR-002 |
| C6 | Invariants live in the constructor. Private field, no back door via serde. | `geometry/src/lib.rs` |
| C7 | Wire format of `PatternBoundary` is a bare `Vec<Point2>`. | `geometry` + Swift mirror |
| C8 | CI gates: fmt, clippy `-D warnings`, test, `cargo deny`, rustdoc. | `.github/workflows/ci.yml` |
| C9 | The crate does not invent geometry. `SeamPath::new` refuses to auto-close. | `curves.rs:143-149` |

**Environment.** Unchanged from the last wave. Rust 1.97.1 pinned, host
`x86_64-pc-windows-msvc`, every cargo invocation through `scripts\cargo.bat` because
Git Bash coreutils `link` shadows MSVC `link.exe`. No macOS toolchain locally; CI
`macos-latest` is the only Swift verification that exists.

**Assumptions.**

| Assumption | State |
|---|---|
| `SeamPath` already validates, serializes through `try_from`, and holds private fields | **VERIFIED** `curves.rs:100-170` |
| A `PatternBoundary` is a closed polygon, so making its closure explicit invents nothing | **VERIFIED** by `PatternBoundary::new` dedup semantics |
| `patal-ffi` never constructs a `PatternPiece` | **VERIFIED** — grep finds none; its three exports take bare `Vec<Point>` |
| `serde_json` is a dev-dependency only in `patal-pattern` | **VERIFIED** `pattern/Cargo.toml` — this rules out a two-pass `Value` peek in the loader |
| `Project` and `ProjectData` both `#[derive(Default)]` | **VERIFIED** `pattern/src/lib.rs:196,212` — a validated tolerance field makes this a trap, see §6 |
| ADR-006 is reserved for the competitive wedge | **VERIFIED** `docs/adr/README.md:17` — this plan is **ADR-007** |
| The Swift package compiles | **UNVERIFIED**. Still no Mac. CI is the only check, and this wave changes Swift again. |

**Unknowns.** All four resolved at the gate: D1 storage shape, D2 tolerance home,
D3 migration, D4 fold-ins. See the Decision Log.

---

## §2 — Domain Mapping

**Problem classification.** Almost purely a **data representation** problem with a
thin functional surface and one concentrated risk. The representation half is the
whole point: what a piece *is* on disk. The functional half is small, three methods
changing signature. The risk is narrow and known, confined to `cut_boundary()`,
because its output is a cutting line. No concurrency, no network, no background work.
The blast radius is small and unusually well bounded, exactly as it was last wave.

One thing distinguishes this wave from the last: **the change is subtractive on the
kernel side.** `PatternBoundary` is not touched at all. Every task either adds a type
above it or changes what `pattern` stores. That is what keeps a CUT-PATH wave cheap.

**Touch-map.**

| Subsystem | Path | Role in this wave |
|---|---|---|
| **geometry: kernel** | `geometry/src/lib.rs` (953 LOC, 31 tests) | **Untouched.** Its tests passing unmodified is the evidence. |
| **geometry: curves** | `geometry/src/curves.rs` (402 LOC) | Gains the boundary lift and join continuity. |
| **pattern** | `pattern/src/lib.rs` (654 LOC, 18 tests) | The centre of the wave. Piece, project, document, migration. |
| **materials** | `materials/src/lib.rs` (339 LOC) | **Untouched.** `MaterialId` is the template `PieceId` copies. |
| **ffi** | `ffi/src/lib.rs` (142 LOC) | **Untouched.** Verified: takes bare `Vec<Point>`, never a piece. |
| **desktop harness** | `apps/desktop/src-tauri/src/lib.rs` | Where the gap is visible, so it is where the fix is proven. |
| **native (Swift)** | `apps/native/Sources/PatalKit/Models/Project.swift` | Decodes `boundary`. Breaks. CI-gated on macOS. See §6 open item. |
| **tests** | `geometry/tests/{curve_oracle,properties}.rs`, `benches/drag_loop.rs` | Extended, not rewritten. |

**Load-bearing invariants in blast radius:**

1. `PatternBoundary` construction-only validity, and its bare `Vec<Point2>` wire format (C7). Neither changes.
2. Correct-or-loud (C1). Every new stored claim is validated or not stored.
3. `SeamPath` construction-only validity (C6), which now has to absorb join continuity without weakening.
4. C9, the refusal to invent geometry. The boundary lift has to be argued against this, not assumed past it.

---

## §3 — Task Decomposition

Legend: ⚠️ **CUT-PATH** marks work whose output becomes a cutting line or a persisted
file.

### §3.1 — `PatternBoundary` → `SeamPath` lift  ⚠️ CUT-PATH
- **Purpose:** the conversion every other task needs. Migration needs it, the piece
  constructor convenience needs it, and Swift's model needs the concept.
- **Depends on:** nothing. This is first.
- **Outputs:** `SeamPath::from_boundary(&PatternBoundary) -> Self` in `curves.rs`.
- Subtasks:
  - Points `p0..pn` become `start = p0`, `segments = [Line{p1} .. Line{pn}, Line{p0}]`.
  - Infallible by construction: a valid `PatternBoundary` already guarantees ≥3 finite
    deduped points, which is exactly what `SeamPath::new` demands. Return `Self`, not
    `Result`. A `Result` here would be a lie about what can fail.
  - **Argue the closing edge against C9.** Appending `Line{p0}` looks like inventing
    geometry, and C9 forbids that. It is not: a `PatternBoundary` is *defined* as a
    closed polygon, so the closing edge already exists and is merely implicit. The lift
    makes it explicit. This is different in kind from `SeamPath::closed` appending an
    edge across a gap the designer left, which is why that function requires the caller
    to opt in. Record the distinction in the doc comment; it will be re-litigated
    otherwise.
  - No float arithmetic anywhere in the lift. That is what makes §3.10's bit-exactness
    property provable rather than approximate.

### §3.2 — Join continuity  ⚠️ CUT-PATH
- **Purpose:** D4-B. A designer dragging a handle across a smooth join breaks tangency
  with nothing to stop them. Intent cannot be re-derived from coordinates, because two
  collinear handles might be coincidence.
- **Depends on:** nothing.
- Design:
  ```rust
  pub enum Join { Corner, Smooth }

  pub struct SeamPath {
      start: Point2,
      segments: Vec<EdgeSegment>,
      joins: Vec<Join>,   // len == segments.len()
  }
  ```
  `joins[i]` describes the join *entering* `segments[i]`, so `joins[0]` is the closure
  join at `start`. A closed path has exactly as many joins as segments, which is why
  this shape has no off-by-one to get wrong.
- Subtasks:
  - **`Smooth` is validated, not merely recorded.** Incoming and outgoing tangents must
    be parallel and same-signed within a relative epsilon. An unvalidated `Smooth` claim
    is a plausible-looking wrong value on a path that feeds the cut line, which C1
    forbids outright. See §6: an earlier draft stored it unchecked and was vetoed.
  - Tangent of `Line{to}` entering at `from` is `to - from`. Tangent leaving a
    `Cubic{c1, ..}` from `p0` is `c1 - p0`.
  - Degenerate handle (`c1 == p0`) leaves the tangent undefined. A `Smooth` claim there
    is refused with `SmoothJoinUndefinedTangent { join }` rather than silently treated
    as a corner.
  - Line-to-cubic smooth joins are legal and must be tested: a straight hem meeting a
    curved side seam smoothly is ordinary pattern making.
  - Serde: `SeamPathData.joins` is `Option<Vec<Join>>`. `None` means all-`Corner` of the
    right length, so a path written before this field existed still loads. A `Some` of
    the wrong length is an error, never padded.

### §3.3 — Grain line  ⚠️ CUT-PATH (persisted)
- **Purpose:** D4-C. Not speculative as first assessed: DXF-AAMA/ASTM defines grain
  line as a specific entity, and export is the next wave. This is the field export
  will need, added while the schema is already moving.
- **Depends on:** nothing.
- Design: `GrainLine { angle_deg: f64, anchor: Point2 }` on
  `PatternPiece.grain: Option<GrainLine>`. `Option` because an unassigned grain line is
  a normal state while designing, same doctrine as `material`.
- Subtasks:
  - Validate finite angle and finite anchor at construction. Private fields, no setter
    that skips the check.
  - **Normalise into `[0, 360)`, not `[0, 180)`.** A grain line is directional, not
    merely axial: napped fabrics (velvet, corduroy) require every piece laid the same
    way, and folding 190° onto 10° would silently destroy that. This is the kind of
    domain fact that looks like a simplification until someone cuts a velvet jacket.
  - Reject non-finite loudly; do not normalise a NaN into something plausible.

### §3.4 — `PieceId`
- **Purpose:** D4-A. ADR-004 records this as the remaining open identity divergence:
  Swift's `PatternPiece` has a `UUID`, Rust's has no identity field at all, which is
  why the piece's document shape is still Swift-to-Swift only.
- **Depends on:** nothing.
- Subtasks:
  - Copy `MaterialId` exactly: UUID-backed, private on the piece, no setter, serde as a
    plain string so Swift's `Foundation.UUID` reads it directly.
  - Add `Project::find_piece_by_id`. Without a reader, `PieceId` is a field that only
    costs bytes; grading and export both index pieces by identity.
  - Keep `find_piece(&str)` by name. It has callers and names stay useful.

### §3.5 — Project-level flatten tolerance
- **Purpose:** D2. The moment a piece holds curves, `cut_boundary()` and
  `total_perimeter_mm()` need a tolerance, and it has to survive the file or a reload
  silently produces a different cut line.
- **Depends on:** nothing.
- Subtasks:
  - `Project.flatten_tolerance_mm: f64`, private, with a validated setter refusing
    non-finite and non-positive values, matching `ToleranceNotPositive` semantics.
  - **Default 0.01mm**, documented against ADR-003's 0.4mm industrial-cutter figure:
    forty times finer than any cutter can execute, and the last wave measured that
    exact tolerance at 7.7% of a 120Hz frame for one piece's full drag path. Affordable,
    with evidence rather than assertion.
  - **Hand-write `Default` for `Project` and `ProjectData`.** See §6: the derived impl
    produces `0.0`, which the validator rejects, so `#[derive(Default)]` would mint
    invalid projects through a path that never calls the setter.
  - No upper bound, deliberately, but see §6 for the watch item.

### §3.6 — `PatternPiece` stores a `SeamPath`  ⚠️ CUT-PATH  — the core change
- **Purpose:** D1. The gap itself.
- **Depends on:** §3.1, §3.5.
- Subtasks:
  - `pub boundary: PatternBoundary` becomes `pub outline: SeamPath`. Public is safe:
    `SeamPath` cannot be constructed invalid, so assignment cannot smuggle in a bad
    value. This is the same reason `boundary` was public.
  - `PatternPiece::new(name, outline: SeamPath)`.
  - `PatternPiece::from_boundary(name, PatternBoundary)` via §3.1, so every existing
    caller migrates in one line.
  - **No persisted polygon and no `#[serde(skip)]` cache in this wave.** The derived
    boundary is computed on demand. A cache is unmeasured optimisation, and the last
    wave established the precedent by dropping the coarse-preview-during-drag strategy
    rather than building it once the benchmark said it was unnecessary. §3.11 measures;
    the cache lands only if the measurement asks for it.
  - `cut_boundary(&self, tolerance_mm)` on the piece, plus
    `Project::cut_boundary(&self, piece)` supplying the project's tolerance. Two
    functions rather than a piece-to-project back-reference: the piece stays testable in
    isolation and the project stays the ergonomic path.
  - **Use `flatten_for_offset(tolerance, allowance)`, not `flatten(tolerance)`.** This
    is a correctness upgrade the wave gets in passing: today `cut_boundary` offsets a
    boundary that was flattened with no knowledge of the impending offset, which is
    precisely the error `flatten_for_offset` exists to prevent.
  - `Project::total_perimeter_mm()` uses plain `flatten` at project tolerance. Nothing
    is being offset, so tightening would be wrong.

### §3.7 — Schema v2 and the migration  ⚠️ CUT-PATH  ⛔ approval node
- **Purpose:** D3. Build the migration mechanism once, on a case where being wrong is
  free.
- **Depends on:** §3.1 through §3.6. It cannot be written until the v2 shape is final,
  which is why the shape freeze is the approval node.
- Subtasks:
  - `SCHEMA_VERSION = 2`.
  - **The loader restructure is the hardest part of this wave.** Today
    `TryFrom<DocumentData>` hard-refuses any mismatch, so there is no dispatch point at
    all. `serde_json` is a dev-dependency only, so a two-pass `Value` peek is not
    available and must not be introduced (it would drag a format-specific dependency
    into a format-agnostic crate).
  - **Rejected: `#[serde(untagged)]`** over v1/v2 variants. It works, but its failure
    message is "data did not match any variant," which is exactly the parse-error-instead-
    of-explanation that ADR-004 says the version field exists to avoid.
  - **Chosen: a hand-written `Deserialize` for `Document`** that reads `schema_version`
    from the map first and dispatches to the right project shape. Roughly 40 lines,
    format-agnostic, and gives an exact message for every failure. Keep `ProjectV1Data`
    private and frozen: it is a historical record, not a live type, and editing it later
    silently changes what old files mean.
  - Migration v1 → v2, as a **pure function** returning a new `Document` or an error.
    Never mutating in place: a half-migrated document that a caller then saves is the
    failure mode that turns a read bug into a write bug.
  - Field mapping: `boundary` lifts via §3.1; `id` is freshly minted (no v1 file
    references pieces by id, so minting is safe); `grain` is `None`; joins default to
    all-`Corner`; project gains the default tolerance.

### §3.8 — Harness update  ⚠️ visual proof
- **Purpose:** the gap was visible in the harness, so the fix is proven there.
- **Depends on:** §3.6, §3.7.
- Subtasks:
  - `save_demo_document` hands `bodice_front()` straight to `PatternPiece::new`. The
    deleted `flatten` call at line 143 is the one-line demonstration.
  - `SaveReport` reports **segment count and cubic count** on the reloaded piece, not
    just byte count. "Reloaded 5 segments, 2 cubic" is the evidence that curves came
    back; a vertex count proves nothing.
  - A command that loads a committed v1 fixture and shows it migrating.
  - `cut_preview` routes through `Project::cut_boundary` instead of flattening inline,
    so the harness exercises the real path rather than a parallel one.

### §3.9 — Swift model: mirror the v2 shape
- **Purpose:** `Project.swift` decodes `boundary` and will not compile against v2. CI is
  gated on it, so leaving it stale is not an option.
- **Depends on:** §3.7.
- **Decided (D5): mirror.** Add `EdgeSegment`, `SeamPath`, `Join`, `GrainLine` and the
  piece's `id` as Codable value types. Roughly 60 lines. See §6 for why this does not
  reopen the last wave's deletion of the Swift offset kernel.
- Subtasks:
  - Value types only. **No algorithms, no geometry, no flattening.** The line this wave
    must not cross is a second implementation of anything that decides where cloth is
    cut. If a Swift function would need a tolerance argument, it does not belong here.
  - `CodingKeys` for snake_case throughout, matching ADR-004's wire contract. `EdgeSegment`
    mirrors the Rust `#[serde(tag = "kind", rename_all = "snake_case")]` shape.
  - `PatternPiece.id` stops being a Swift invention and becomes the engine's `PieceId`,
    exactly as `material` became `MaterialId` last wave. Delete the "Swift-to-Swift only"
    comment at `Project.swift:82-88`; §3.4 is what retires it.
  - **Swift does not re-validate.** No Swift-side tangent check for `Join::Smooth`, no
    closure check for `SeamPath`. Rust owns validation; a second validator is a second
    implementation that can disagree, which is the exact failure the kernel deletion
    removed. Swift decodes what the engine wrote and trusts it.
  - Round-trip test in `PatalKitTests`: decode a committed v2 fixture, re-encode, compare.
    Same fixture the Rust tests use, so the two languages are pinned to one file rather
    than to each other.

### §3.10 — Properties, oracles, fixtures  ⚠️ CUT-PATH
- **Depends on:** the tasks each one covers.
- Subtasks:
  - **proptest, the headline property:** `lift(b).flatten(t)` is *bit-identical* to `b`,
    for every valid boundary `b` and every valid tolerance `t`. Bit-identical rather
    than within-epsilon, because §3.1 does no float arithmetic. If this ever needs an
    epsilon, the lift has acquired a bug.
  - Migration losslessness on the committed v1 fixture.
  - Tolerance persistence: save → load → `cut_boundary` yields identical points.
  - `Join::Smooth` with non-collinear handles is rejected; line-to-cubic smooth joins
    are accepted; a degenerate handle with a `Smooth` claim errors.
  - Grain angle: 190° stays 190°, non-finite is refused.
  - The 31 `geometry/src/lib.rs` tests pass **unmodified**.
  - Expect `properties.proptest-regressions` to need regeneration once generators change.
    Noise, not a defect, but it will look like one in review.

### §3.11 — Benchmarks
- **Depends on:** §3.6.
- Extend `benches/drag_loop.rs` to measure through `PatternPiece::cut_boundary`, so the
  no-cache decision in §3.6 is measured rather than assumed. Add a
  `total_perimeter_mm` case at 50 pieces: that is the call that flattens every piece
  with no cache behind it, and it is the one place the decision could be wrong.

### §3.12 — ADR-007 and doc close-out
- **ADR-007 (new):** what a pattern piece stores. Records D1 through D4, the C9
  argument for the lift, the validated-`Smooth` veto, the tolerance default with its
  measurement, and the loader-dispatch rejection of `untagged`.
- Close ADR-004's two open items: the flattened-boundary note (this wave) and the piece
  identity divergence (§3.4).
- ADR-003 gains a back-reference: the two-layer split now reaches the document.
- README and vault status refreshed.

---

## §4 — Dependency + Ordering (DAG)

```
§3.1 (lift, ⚠️) ──┐
§3.2 (joins, ⚠️) ─┤
§3.3 (grain, ⚠️) ─┼──▶ §3.6 (piece stores SeamPath, ⚠️) ──▶ §3.7 (v2 + migration, ⚠️⛔)
§3.4 (PieceId) ───┤            ▲                                    │
§3.5 (tolerance) ─┘────────────┘                                    ├──▶ §3.8 (harness)
                                                                    └──▶ §3.9 (Swift, ⛔)
§3.6 ──▶ §3.11 (benchmarks)
§3.10 (tests) threads through every task it covers
§3.12 (ADR-007) ── after the decisions land, parallel with code
```

**Ordered sequence:** `{§3.1, §3.2, §3.3, §3.4, §3.5} → §3.6 → §3.7 → {§3.8, §3.9}`,
with §3.10 folded into each task and §3.11, §3.12 trailing.

**Parallelizable set.** §3.1 through §3.5 have no mutual dependency at all. That is
unusual and worth exploiting: five independent additive changes land before anything
structural moves. If this wave is split across sessions, that boundary is the clean
cut.

**Approval nodes (one-way doors):**
- ⛔ **§3.7 v2 shape freeze.** Once the migration is written against a shape, changing
  the shape means changing the migration. This is the moment to be sure, and it is
  cheap to pause here because §3.1-§3.6 are all additive or internal.
- ⛔ **§3.9 Swift: mirror or delete.** Operator call, framed in §6.

**Ordering finding.** The natural instinct is to write the migration first, since it
sounds foundational. It is the opposite: the migration is the *last* thing that can be
written, because it encodes the final v2 shape. Writing it early guarantees rewriting
it. §3.7 sits behind everything for that reason.

---

## §5 — Execution Specification

### §5.1 — spec for §3.1 (lift)  ⚠️ CUT-PATH
- **Method:** direct structural conversion, zero arithmetic.
- **Artifacts:** `SeamPath::from_boundary`, its C9 doc argument, the bit-exactness property.
- **Validation:** proptest bit-identity across every generated boundary and tolerance;
  the 31 kernel tests unmodified.
- **Failure modes:** the closing edge is read as invented geometry in review and gets
  "fixed" into an auto-close opt-in, breaking the infallibility.
- **Fallback:** none needed. If bit-identity fails, the lift has arithmetic in it that
  should not be there, and the fix is removal, not tolerance.

### §5.2 — spec for §3.2 (joins)  ⚠️ CUT-PATH
- **Method:** tangent comparison at construction, relative epsilon scaled to the
  coordinates involved, mirroring `CLOSURE_SNAP_RELATIVE` at `curves.rs:65`.
- **Artifacts:** `Join`, `SeamPath.joins`, `SmoothJoinUndefinedTangent` and a
  non-collinear variant, `Option<Vec<Join>>` wire shape.
- **Validation:** smooth-claim rejection tests; line-to-cubic acceptance; missing
  `joins` key loads as all-`Corner`; wrong-length `joins` errors.
- **Failure modes:** the epsilon is too tight and refuses joins a designer legitimately
  drew; too loose and it certifies a visible kink as smooth.
- **Fallback:** if the epsilon proves contentious, widen it and pin the chosen value in
  a named test with the reasoning. Never drop the check to make a case pass.

### §5.3 — spec for §3.6 (the piece)  ⚠️ CUT-PATH
- **Method:** field replacement plus a derived accessor. No cache.
- **Artifacts:** `outline: SeamPath`, `from_boundary`, `cut_boundary(tolerance_mm)`,
  `Project::cut_boundary`, updated `total_perimeter_mm`.
- **Validation:** all 18 pattern tests updated only where the type changed, never where
  behaviour should have held; `apps/desktop` compiles under `-D warnings`; the
  `flatten_for_offset` upgrade is asserted by a test that fails under plain `flatten`.
- **Failure modes:** a caller passes the project tolerance to `flatten_for_offset` and
  the offset then exceeds what the curve can give, producing a legitimate
  `OffsetSelfIntersects`. **That is correct behaviour, not a regression** — the same
  finding the last wave recorded, where a shape succeeds at 0.01mm and fails at 0.001mm
  because a chord next to a sharp corner becomes shorter than the allowance.
- **Fallback:** do not weaken the check. The product answer is the harness message, and
  it is already wired.

### §5.4 — spec for §3.7 (v2 and migration)  ⚠️ CUT-PATH
- **Method:** hand-written `Deserialize` dispatching on `schema_version`; migration as a
  pure function.
- **Artifacts:** `SCHEMA_VERSION = 2`, frozen private `ProjectV1Data`, `migrate_v1`, the
  committed v1 fixture.
- **Validation:** fixture migrates; migrated outline flattens bit-identical to the v1
  boundary; a v3 document is refused with a readable message, not a parse error; a
  malformed v1 fails at v1's own validator rather than being coerced.
- **Failure modes:** the hand-written impl drifts from the derived one for v2 documents
  and silently accepts something the derive would reject.
- **Fallback:** assert derive-equivalence for v2 in a test — round-trip a v2 document
  through both paths and compare. Cheap, and it pins the impl to the derive permanently.

### §5.5 — spec for §3.10 (tests)  ⚠️ CUT-PATH
- **Method:** proptest for the lift, fixtures for the migration, named regressions for
  everything a property finds.
- **Validation:** zero panics across all generators; every shrunk failure promoted into a
  named test before being fixed.
- **Failure modes:** a property discovers a real defect in shipped code, which is the
  point of the exercise. The last wave's suite found two.

---

## §6 — Risk + Ambiguity Audit (self-adversarial)

### CRITIC pass

**What I left out on the first pass and had to add back:**

- **`#[derive(Default)]` on `Project` and `ProjectData`** (`pattern/src/lib.rs:196,212`).
  Adding a validated `flatten_tolerance_mm` makes the derived impl produce `0.0`, which
  the setter would reject, so `Project::default()` would mint an invalid project through
  a path that never calls the validator. This is a C1 violation created by a derive
  nobody would think to look at. §3.5 hand-writes both. **Verified present in the current
  tree; this is the single most likely silent break in the wave.**
- **`Project::default()` has callers by way of `ProjectData`'s `#[serde(default)]` on
  `materials`.** Same trap, one level down, and it fires during deserialization rather
  than at an obvious call site.
- **The migration must be pure.** First draft mutated a `Document` in place. A failed
  migration would then leave a half-migrated document that a caller could save, turning
  a read bug into a write bug. Now specified as a function returning a new document.
- **`PieceId` needs a reader.** §3.4 originally added the field and nothing else. A field
  with no lookup is bytes on disk and nothing more; `find_piece_by_id` is what makes it
  real, and grading and export both index by identity.
- **Grain line direction, not axis.** First draft normalised into `[0, 180)` as a tidy
  simplification. Napped fabrics make grain direction load-bearing, so folding 190° onto
  10° would silently destroy the constraint that makes a velvet jacket cuttable.

**Assumptions not verified, and what happens if each is wrong:**

- *The Swift package compiles.* Still never built locally. Same known-unknown as last
  wave, and this wave changes Swift again. CI is the only check. Not a plan risk so much
  as a plan output.
- *No upper bound on flatten tolerance.* A tolerance of 1e9 turns every curve into a
  straight line. It is not *wrong* in the correct-or-loud sense, just useless, and
  inventing a bound means inventing a number. **Watch item, deliberately unresolved:**
  revisit if a real user ever sets one, and do not guess a limit now.
- *No cache is the right call.* Rests on last wave's measurement of one piece. §3.11's
  50-piece `total_perimeter_mm` case is where it could be wrong, and that case exists
  precisely because I do not trust the extrapolation.

**Worst case if the plan is wrong overall.** The v2 shape is wrong and needs a v3. That
cost is now materially lower than it was before this wave, because §3.7 builds the
migration machinery. **D3 is the hedge that makes being wrong about D1 survivable**,
which is the strongest argument for having taken it.

### RISK-AGENT pass (rebound to Pātāl's rules)

| Rule | Check | Verdict |
|---|---|---|
| C1 correct-or-loud | Tolerance validated; `Smooth` validated; grain validated; migration total; unresolved ids still error. | PASS |
| C2 no unsafe | No task introduces `unsafe`. | PASS |
| C3 core purity | Grain line is a domain concept, not a UI one. No platform types. | PASS |
| C4 no per-frame FFI | FFI surface unchanged. Verified: it never constructs a piece. | PASS |
| C6 constructor invariants | `PieceId` private no setter; tolerance private with validated setter; `SeamPath` serde still via `try_from`. | PASS |
| C7 wire format | `PatternBoundary` untouched. Every change is above it. | PASS |
| C9 no invented geometry | The lift's closing edge is argued, not assumed. See §3.1. | PASS |
| Cut-path integrity | The 953-LOC kernel is not modified. All 31 tests pass unmodified. | PASS |

**Verdict: APPROVED.**

**One revision was forced during this pass.** An earlier draft of §3.2 stored `Join::Smooth`
as an unchecked flag, on the reasoning that continuity is designer intent rather than
geometry. That is wrong on the file's own terms: a `Smooth` claim the coordinates
contradict is a plausible-looking wrong value sitting on a path that feeds the cut line,
which is exactly what C1 forbids and exactly the class of defect the last wave fixed in
`offset()`. **VETOED** and replaced with construction-time tangent validation. If a
claim cannot be checked, it should not be stored.

### ⛔ Open decision the operator owns: §3.9, Swift mirror or delete

Framed rather than decided, because it re-opens a question the last wave answered in the
other direction.

- **Mirror it.** Add `EdgeSegment`, `SeamPath`, `Join`, `GrainLine`, `PieceId` as Codable
  value types, roughly 60 lines, no algorithms. Unlike the deleted offset kernel there is
  no divergent-behaviour risk: a Codable struct either decodes the snake_case shape or it
  does not.
- **Delete it.** The A1 argument from last wave applies unchanged — Swift's
  `PatternPiece` has no reachable consumer, there is no Xcode project, and it is
  Swift-to-Swift only by its own admission. Deleting the models costs nothing today and
  removes the tax permanently.

**Recommendation: mirror.** The deleted code was a second *implementation of geometry*;
this is a data model, and the distinction is what makes it cheap. But the counter-argument
is genuinely strong and it is your call, not mine. Leaving it stale is the one answer that
is definitely wrong, since CI is gated on it.

---

## §7 — Final Assembly

**Build order.**

1. §3.1 lift, with the C9 argument in the doc comment. → bit-exactness property green.
2. §3.2 joins, validated. → smooth-claim rejection tests green.
3. §3.3 grain line. → 190° survives normalisation.
4. §3.4 `PieceId` + `find_piece_by_id`. → ADR-004's identity divergence closed.
5. §3.5 project tolerance + **hand-written `Default`**. → `Project::default()` is valid.
6. §3.6 piece stores `SeamPath`; `flatten_for_offset` upgrade. → 18 pattern tests green.
7. ⛔ **v2 shape freeze.** Operator sign-off before the migration is written.
8. §3.7 hand-written `Deserialize`, frozen `ProjectV1Data`, `migrate_v1`. → fixture migrates losslessly.
9. §3.8 harness: `flatten` call deleted, segment count reported, v1 fixture command. → curves visibly return.
10. ⛔ §3.9 Swift: mirror or delete, per the §6 decision. → `native` job green.
11. §3.11 benchmarks incl. the 50-piece case. → the no-cache call is measured.
12. §3.12 ADR-007, ADR-004 open items closed, README and vault refreshed.

**Acceptance criteria.**

- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings` clean
- [ ] `cargo test --workspace --locked` green, count reported and **> 89**
- [ ] `cargo deny check` clean
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` clean
- [ ] `apps/desktop` compiles under `-D warnings`
- [ ] CI `native` job green
- [ ] All 31 pre-existing `geometry/src/lib.rs` tests pass **unmodified**
- [ ] proptest: `lift(b).flatten(t)` bit-identical, zero panics across all generators
- [ ] v1 fixture migrates; migrated outline flattens bit-identical to the v1 boundary
- [ ] Save → load → `cut_boundary` yields identical points
- [ ] `Join::Smooth` with non-collinear handles is refused
- [ ] `Project::default()` produces a valid tolerance
- [ ] No `PatternBoundary` appears in any serialized `.patal`

**Deliverables.** This blueprint; ADR-007; a committed v1 fixture; the migration path;
ADR-004's two open items closed; harness proof that curves survive a round trip.

---

## Decision Log

| D# | Question | Chosen | Why |
|---|---|---|---|
| D1 | What a piece stores | **`SeamPath` only, polygon derived** | Matches ADR-003's authored/manufactured split exactly and ADR-004's one-source-of-truth doctrine. Persisting both would move the stale-copy bug class from materials to geometry. |
| D2 | Where tolerance lives | **Project-level, persisted** | Tolerance describes the manufacturing target. Unpersisted, a reload silently produces a different cut line, which C1 forbids. |
| D3 | Schema v2 migration | **Build a real v1→v2 migration** | The mechanism will never be cheaper to prove. It is also the hedge that makes being wrong about D1 survivable. |
| D4 | Fold-ins | **PieceId + join continuity + grain line** | All three are storage questions. Each one deferred is a candidate schema v3 for a field already known to be wanted. |

## Revision Log

| # | Section | Change | Trigger |
|---|---|---|---|
| 1 | §3.2 | Unchecked `Smooth` flag replaced by construction-time tangent validation | §6 RISK-AGENT veto on C1 |
| 2 | §3.3 | Grain normalisation `[0,180)` → `[0,360)` | CRITIC pass: napped fabrics make direction load-bearing |
| 3 | §3.5 | Hand-written `Default` added for `Project`/`ProjectData` | CRITIC pass: derived `Default` mints an invalid tolerance |
| 4 | §3.7 | `#[serde(untagged)]` rejected for hand-written `Deserialize` | `serde_json` is dev-only; untagged errors defeat the version field's purpose |
| 5 | §4 | Migration moved from first to last | It encodes the final v2 shape, so writing it early guarantees rewriting it |
