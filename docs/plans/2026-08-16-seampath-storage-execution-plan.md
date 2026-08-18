---
title: SeamPath Storage — Execution Plan
tags: [plan, execution]
updated: 2026-08-16
---

# SeamPath Storage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps
> use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the authored curve the thing a `.patal` file stores, so a saved piece can be
edited back into curves instead of arriving as a frozen polygon — and build the schema
migration mechanism once, while the only file that exists is disposable.

**Architecture:** Five independent additive changes land first (`Edge` container, the
polygon→path lift, grain line, `PieceId`, project flatten tolerance). Then one structural
change swaps `PatternPiece.boundary: PatternBoundary` for `outline: SeamPath`, which breaks
`patal-export` and the harness in the same commit and is repaired there. Then the v2 shape
is frozen by the operator, the migration is written against the frozen shape, and the
harness and Swift model follow. The polygon is never persisted again — it is derived on
demand at the project's tolerance.

**Tech Stack:** Rust 1.97.1 (pinned, `rust-toolchain.toml`), serde 1 with
`float_roundtrip`, uuid v4, proptest 1, criterion 0.8. Swift 5 / SwiftPM for
`apps/native`. Tauri 2 for `apps/desktop`. No new third-party dependencies except adding
the existing workspace `uuid` to `patal-pattern`.

**Spec:** [`docs/plans/2026-08-13-seampath-storage-ultraplan.md`](2026-08-13-seampath-storage-ultraplan.md)
(revision 6). The plan argues from that blueprint; executors read both. Where this plan
departs from it, the departure is marked **DEVIATION** and carries its reason.

---

## Global Constraints

Every task's requirements implicitly include this section. Values copied verbatim from
the blueprint §1 unless marked.

| # | Rule |
|---|---|
| C1 | Correct or loud. Never return a plausible-looking number from a fallible op. |
| C2 | `#![forbid(unsafe_code)]` stays in every crate. |
| C3 | The core imports no platform UI types. |
| C4 | The render loop never crosses FFI per frame. |
| C5 | `Pātāl` in prose and UI, `Patal` in anything a toolchain touches. |
| C6 | Invariants live in the constructor. Private field, no back door via serde. |
| C7 | Wire format of `PatternBoundary` is a bare `Vec<Point2>`. Unchanged by this wave. |
| C8 | CI gates: fmt, clippy `-D warnings`, test, `cargo deny`, rustdoc. |
| C9 | The crate does not invent geometry. `SeamPath::new` refuses to auto-close. |
| C10 | **Build only via `scripts\cargo.bat`.** Git Bash's coreutils `link` shadows MSVC `link.exe` and rustc's own error names the wrong cause. From Git Bash: `cmd //c 'C:\Users\User\patal\scripts\cargo.bat <args>'`. |
| C11 | One implementation of the cut line. `CutLine` has a private field and no public constructor; nothing outside `patal-pattern` can mint one. |
| C12 | **The 953-LOC kernel `geometry/src/lib.rs` is not modified.** Its 31 tests passing unmodified is the evidence. Adding error variants to `GeometryError` is the one permitted exception — see Task 2. |

**Verification command set.** Run all five before declaring any task done:

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat fmt --check'
cmd //c 'C:\Users\User\patal\scripts\cargo.bat clippy --workspace --all-targets --locked -- -D warnings'
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --workspace --locked'
cmd //c 'C:\Users\User\patal\scripts\cargo.bat doc --no-deps --workspace --locked'
PATAL_CARGO_DIR=C:\Users\User\patal\apps\desktop\src-tauri cmd //c 'C:\Users\User\patal\scripts\cargo.bat clippy --all-targets --locked -- -D warnings'
```

`RUSTDOCFLAGS=-D warnings` must be exported from bash for the doc gate; `set VAR=x` inside
`cmd //c` silently does not reach the process.

**Baseline at plan time.** `main` = `4abf281` plus PRs #5 and #6 pending merge. Working tree
green: fmt clean, clippy clean, **136 tests pass** (135 unit and integration, plus one
doc-test).

**Corrected baseline (2026-08-16).** 136 is the count on `tile-count-saturates`, which is
not merged. A wave branch cut from `main` starts two lower, so Task 1 finished at **138**,
not the 139 this plan projected. Task 2's projected 145 is therefore 144. From Task 3
onward the projections are correct again, because Task 3 landed three properties rather
than two — see that task. Every count from Task 3 to Task 6 was hit exactly: 147, 154, 157,
161.

**Execution status (2026-08-17).** Tasks 1–7 are complete and committed on
`seampath-storage-wave`, all six gates green at each one:

| Task | Commit | Tests after |
|---|---|---|
| 1 — `Edge` container | `16188a4` | 138 |
| 2 — `Smooth` validated | `a60ee9f` | 144 |
| 3 — the lift | `0d517ba` | 147 |
| 4 — `GrainLine` | `0a9ed0f` | 154 |
| 5 — `PieceId` | `9ec6a20` | 157 |
| 6 — flatten tolerance | `c45d22e` | 161 |
| 7 — the piece stores a path | `c6ac313` | **168** |

**The hashes above are not the ones this plan carried yesterday.** The branch was rebased
onto `main` on 2026-08-17 after PRs #5 and #6 merged, so every commit was rewritten. The
old hashes (`1d5e5d5` … `c071f47`) are unreachable; these are the live ones.

**Baseline moved, and the projections after it move with it.** Merging PR #5
(`tile-count-saturates`) brought its 2 tests onto `main`, so the wave's baseline went 161 →
163 without any wave work. Task 7's projected 166 is therefore **168**, and it was hit
exactly. Every later projection in this plan is understated by the same 2 — read Task 8's
"expect N" as N + 2, and re-derive from the live count rather than from the plan.

The wave now sits at the ⛔ **v2 shape freeze** gate, below Task 7. That is the next thing
needing an operator answer, and it is the one-way door.

---

## Reconciliation findings — read before Task 1

The blueprint was written 2026-08-13 against `main` @ `dc509eb`. The tree has moved. These
are the differences that change the work, each verified against the tree today rather than
assumed.

### R1 — `patal-export` is in the blast radius and the blueprint does not mention it

**The blueprint's §2 touch-map has no row for `patal-export`,** because the crate did not
exist when it was written — export merged 2026-08-14 via PR #1, a day later. It is not an
untouched crate like `patal-ffi`; it is broken by §3.6 in four places:

| Site | Breaks how |
|---|---|
| `engine/crates/export/src/lib.rs:124` | `piece.cut_boundary()` — gains a required `tolerance_mm` argument |
| `engine/crates/export/src/lib.rs:134` | `BoundsMm::of_boundary(&piece.boundary)` — the field is renamed and retyped |
| `engine/crates/export/src/lib.rs:149` | `sewing: piece.boundary.clone()` — same |
| `engine/crates/export/src/lib.rs:489,534` | `PatternPiece::new(name, boundary)` in test helpers — the constructor's second parameter changes type |

Export is 1,923 LOC with its own test suite and a **byte-compared golden PDF**. Task 7
handles it, and it is the largest single piece of unplanned work in this wave.

### R2 — `Project::total_perimeter_mm()` must become fallible

`pattern/src/lib.rs:373` is `self.pieces.iter().map(|p| p.boundary.perimeter()).sum()`. Once
a piece stores a `SeamPath`, the perimeter requires a flatten, and flatten returns
`Result`. The blueprint's §3.6 says "uses plain `flatten` at project tolerance" but does not
say the signature changes. It must: `-> Result<f64, PatternError>`.

### R3 — `patal-pattern` does not depend on `uuid`

§3.4 says "copy `MaterialId` exactly". `MaterialId` lives in `patal-materials`, which has the
`uuid` dependency. `PieceId` belongs on the piece, in `patal-pattern`, whose `Cargo.toml`
lists only `patal-geometry`, `patal-materials` and `serde`. Task 5 adds
`uuid = { workspace = true }` to it.

### R4 — Build §3.2 before §3.1 **DEVIATION**

The blueprint's §7 build order runs the lift (§3.1) first, then the `Edge` container (§3.2).
The §4 DAG says the two have no mutual dependency, so the order between them is free. Doing
the lift first means writing it against `segments: Vec<EdgeSegment>` and rewriting it hours
later against `edges: Vec<Edge>`. **This plan builds the container first**, so the lift is
written once, against the final shape. Nothing else in the DAG moves.

### R5 — The blueprint's `SeamPath` line references are stale but its claims hold

The blueprint cites `curves.rs:143-149` for C9 and `curves.rs:65` for `CLOSURE_SNAP_RELATIVE`.
Verified today: `CLOSURE_SNAP_RELATIVE` is still at `curves.rs:65`; the C9 argument is at
`curves.rs:143-149`. `SeamPath` is `curves.rs:118-123`, `segments()` at `curves.rs:226-228`.
The four `segments()` call sites are all in `geometry/tests/curve_oracle.rs:493-498`, exactly
as claimed. `patal-ffi` still never constructs a `PatternPiece`.

### R6 — The v1→v2 loader cannot read `schema_version` first **DEVIATION**

§3.7 specifies "a hand-written `Deserialize` for `Document` that reads `schema_version` from
the map first and dispatches to the right project shape." That requires either buffering the
map or assuming key order. JSON object key order is not guaranteed by the format, `serde_json`
is a dev-dependency only so `Value` is unavailable, and `serde::__private::de::Content` is
private API this crate must not reach into.

**This plan uses an order-independent equivalent:** one private, frozen, version-tolerant
data shape whose version-specific fields are `Option`, plus a strict `TryFrom` that dispatches
on `schema_version` and *rejects* a document carrying the wrong version's fields. It meets
every validation item in §5.4 — exact messages, no `untagged`, frozen historical shape,
migration as a pure function — without depending on key order. Task 9 spells it out.

---

## File Structure

| File | Responsibility | Change |
|---|---|---|
| `engine/crates/geometry/src/lib.rs` | The polygon kernel. | **Error variants only.** No logic. C12. |
| `engine/crates/geometry/src/curves.rs` | Authored curves. | Gains `Join`, `Edge`, `SeamPath::with_joins`, `SeamPath::from_boundary`; `segments()` → `edges()`. |
| `engine/crates/geometry/tests/curve_oracle.rs` | Closed-form oracle. | 4 mechanical call-site updates; new join tests. |
| `engine/crates/geometry/tests/properties.rs` | Property suite. | Gains the lift bit-exactness property. |
| `engine/crates/geometry/benches/drag_loop.rs` | Drag-loop budget. | Gains a `cut_boundary` case and a 50-piece perimeter case. |
| `engine/crates/pattern/src/lib.rs` | Piece, project, document, migration. | The centre of the wave. |
| `engine/crates/pattern/src/grain.rs` | **New.** `GrainLine` and its validation. | Split out so `lib.rs` does not grow past ~900 lines. |
| `engine/crates/pattern/src/migrate.rs` | **New.** Version-tolerant load and `migrate_v1`. | Split out: it is a frozen historical record and must be readable as one. |
| `engine/crates/pattern/Cargo.toml` | | Adds `uuid`. |
| `engine/crates/export/src/lib.rs` | Tiled PDF. | `export_tiled_pdf` becomes project-aware. R1. |
| `apps/desktop/src-tauri/src/lib.rs` | The harness. | The `flatten` call at line 143 is deleted; `SaveReport` reports curves. |
| `apps/native/Sources/PatalKit/Models/Geometry.swift` | Swift geometry mirror. | Gains `EdgeSegment`, `Edge`, `Join`, `SeamPath`. |
| `apps/native/Sources/PatalKit/Models/Project.swift` | Swift piece mirror. | `boundary` → `outline`; `id` becomes the engine's. |
| `fixtures/v1-bodice.patal` | **New.** Frozen v1 document. | Read by Rust migration tests. |
| `fixtures/v2-bodice.patal` | **New.** Frozen v2 document. | Read by **both** Rust and Swift, so the two languages pin to one file rather than to each other. |
| `docs/adr/ADR-007-what-a-pattern-piece-stores.md` | **New.** | D1–D4, the C9 argument, the `Edge` container rationale. |

---

## Decisions this plan needs from the operator

Three, not two. The blueprint frames two; R1 forces a third.

- **✅ D6 — export's public signature. ANSWERED 2026-08-17: option A**, project-aware.
  Shipped in Task 7.
- **⛔ §3.7 — the v2 shape freeze.** Before Task 8. One-way door: once the migration is
  written against a shape, changing the shape means changing the migration. **This is the
  live gate as of 2026-08-17.**
- **⛔ §3.9 — Swift: mirror or delete.** Before Task 10. Blueprint §6 recommends mirror.

Tasks 1–6 are additive or internal and none of them depends on these answers. Work can start
immediately and stop cleanly at Task 7.

*Corrected 2026-08-17.* The two gate references above said "before Task 9" and "before
Task 11". Both were off by one against this plan's own headings — the freeze gate sits
immediately before **Task 8**, and the Swift gate immediately before **Task 10**. Fixed
here because an operator reading only this section would have believed a task's worth of
slack existed that does not.

---

### Task 1: The `Edge` container (§3.2, container half)

Mechanical and behaviour-preserving. Every path built through `new` or `closed` comes out
all-`Corner`, exactly as today. No join validation is added here — Task 2 does that — so
this task's whole claim is "the workspace compiles and every existing test still passes
against a changed internal shape."

**Files:**
- Modify: `engine/crates/geometry/src/curves.rs:100-140` (wire shape + struct), `:222-332` (accessors)
- Modify: `engine/crates/geometry/tests/curve_oracle.rs:493-498` (4 call sites)

**Interfaces:**
- Consumes: nothing.
- Produces: `pub enum Join { Corner, Smooth }` with `Default = Corner`; `pub struct Edge`
  with `Edge::new(EdgeSegment, Join) -> Edge`, `Edge::corner(EdgeSegment) -> Edge`,
  `Edge::geometry(&self) -> EdgeSegment`, `Edge::join(&self) -> Join`,
  `Edge::end(&self) -> Point2`; `SeamPath::edges(&self) -> &[Edge]`;
  `SeamPath::with_joins(Point2, Vec<Edge>) -> Result<SeamPath, GeometryError>`.
  `SeamPath::new` and `SeamPath::closed` keep their exact current signatures.
  `SeamPath::segments()` is **removed**.

- [ ] **Step 1: Write the failing tests**

Append to `engine/crates/geometry/tests/curve_oracle.rs`, and extend that file's existing
import to bring in `Edge` and `Join`:

```rust
#[test]
fn every_edge_from_the_plain_constructor_is_a_corner() {
    let start = Point2::new(0.0, 0.0);
    let path = SeamPath::new(
        start,
        vec![
            EdgeSegment::Line { to: Point2::new(10.0, 0.0) },
            EdgeSegment::Line { to: Point2::new(10.0, 10.0) },
            EdgeSegment::Line { to: start },
        ],
    )
    .expect("a triangle closes");

    assert_eq!(path.edges().len(), 3);
    assert!(path.edges().iter().all(|e| e.join() == Join::Corner));
    assert_eq!(
        path.edges()[0].geometry(),
        EdgeSegment::Line { to: Point2::new(10.0, 0.0) }
    );
    assert_eq!(path.edges()[2].end(), start);
}

#[test]
fn an_edge_is_a_nested_object_on_the_wire_not_a_flat_one() {
    // The nesting is the point. When per-edge allowance and fold arrive they
    // are siblings of `join`, while `to` and `c1` are the geometry itself. A
    // flat map puts them in one bag as though they were the same kind of
    // thing, and the shape stops teaching the distinction the container was
    // chosen to make.
    let start = Point2::new(0.0, 0.0);
    let path = SeamPath::new(
        start,
        vec![
            EdgeSegment::Line { to: Point2::new(1.0, 0.0) },
            EdgeSegment::Line { to: start },
        ],
    )
    .expect("closes");

    let json = serde_json::to_value(&path).expect("serializes");
    let edge = &json["edges"][0];
    assert_eq!(edge["geometry"]["kind"], "line");
    assert_eq!(edge["join"], "corner");
    assert!(
        edge.get("to").is_none(),
        "geometry must not be flattened into the edge"
    );
}

#[test]
fn an_edge_with_no_join_key_loads_as_a_corner() {
    // `Corner` is the absence of a claim, so omitting it cannot manufacture
    // one. A hand-edited `.patal` is explicitly in scope for this repo.
    let json = r#"{
        "start": {"x": 0.0, "y": 0.0},
        "edges": [
            {"geometry": {"kind": "line", "to": {"x": 1.0, "y": 0.0}}},
            {"geometry": {"kind": "line", "to": {"x": 0.0, "y": 0.0}}}
        ]
    }"#;
    let path: SeamPath = serde_json::from_str(json).expect("loads without a join key");
    assert!(path.edges().iter().all(|e| e.join() == Join::Corner));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-geometry --test curve_oracle'
```

Expected: FAIL to compile — "cannot find type `Edge` in this scope", "no method named
`edges` found".

- [ ] **Step 3: Add `Join` and `Edge`**

Insert into `engine/crates/geometry/src/curves.rs` immediately after the `EdgeSegment` impl
block (after line 98):

```rust
/// How an edge meets the edge before it.
///
/// A *claim about intent*, which is why it is stored rather than re-derived:
/// two collinear handles might be coincidence, and a designer who wants
/// tangency preserved through an edit needs somewhere to say so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Join {
    /// No continuity claim. The *absence* of a claim, which is why it is the
    /// serde default: omitting the key cannot manufacture a claim the
    /// coordinates contradict.
    #[default]
    Corner,
    /// The tangents on either side of this join are parallel and same-signed.
    /// Validated at construction; see [`SeamPath::with_joins`].
    Smooth,
}

/// One authored edge: its geometry, and how it meets the edge before it.
///
/// # Why a struct around one field
///
/// `join` is the *first* per-edge attribute, not the only one. The primitive
/// census identifies three more that are attributes of an edge — per-edge seam
/// allowance (P-03), fold edges (P-05) and notch anchors (P-13) — and per-edge
/// allowance is a fold-in rather than a maybe, because a neckline is finished
/// at 6mm and a hem is turned at 40mm.
///
/// Each of those arriving as its own array parallel to the geometry adds a
/// `len ==` invariant and one more thing every edit that splits a segment must
/// keep in step. Four arrays is four chances to get it wrong, and the fourth
/// gets it wrong on the path that feeds the cut line. This struct is what makes
/// adding the second attribute a field rather than a schema migration.
///
/// Do not flatten it back. See `docs/adr/ADR-007-what-a-pattern-piece-stores.md`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    geometry: EdgeSegment,
    #[serde(default)]
    join: Join,
}

impl Edge {
    pub fn new(geometry: EdgeSegment, join: Join) -> Self {
        Self { geometry, join }
    }

    /// An edge making no continuity claim — what every edge built through
    /// [`SeamPath::new`] or [`SeamPath::closed`] is.
    pub fn corner(geometry: EdgeSegment) -> Self {
        Self { geometry, join: Join::Corner }
    }

    pub fn geometry(&self) -> EdgeSegment {
        self.geometry
    }

    pub fn join(&self) -> Join {
        self.join
    }

    /// Where this edge ends. Delegates to the geometry.
    pub fn end(&self) -> Point2 {
        self.geometry.end()
    }
}
```

- [ ] **Step 4: Swap the container**

Replace `curves.rs:100-140` — the `SeamPathData` struct, the `SeamPath` struct and both
conversion impls — with:

```rust
/// The wire shape of a [`SeamPath`]: exactly its fields, so a document is
/// readable without knowing anything about this type's invariants.
#[derive(Serialize, Deserialize)]
struct SeamPathData {
    start: Point2,
    edges: Vec<Edge>,
}

/// A closed, authored outline: where it starts, and the edges that walk it
/// back around to that point.
///
/// # Invariants
///
/// Construction is the only way in, exactly as with [`PatternBoundary`]:
/// every control point is finite, there is at least one edge, the last edge
/// ends precisely where the path started, and every `Join::Smooth` claim is
/// backed by the coordinates. `serde` routes through
/// [`SeamPath::with_joins`] via `try_from`, so a hand-edited file cannot
/// smuggle in an open, non-finite, or falsely-smooth path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "SeamPathData", into = "SeamPathData")]
pub struct SeamPath {
    start: Point2,
    edges: Vec<Edge>,
}

impl From<SeamPath> for SeamPathData {
    fn from(path: SeamPath) -> Self {
        Self { start: path.start, edges: path.edges }
    }
}

impl TryFrom<SeamPathData> for SeamPath {
    type Error = GeometryError;

    fn try_from(data: SeamPathData) -> Result<Self, Self::Error> {
        Self::with_joins(data.start, data.edges)
    }
}
```

- [ ] **Step 5: Rewrite the constructors and accessors**

Replace the body of `SeamPath::new` (`curves.rs:150-170`) and add `with_joins` beside it.
`new` and `closed` keep their signatures exactly, so all 14 existing call sites compile
untouched:

```rust
    /// Validates a closed path whose edges make no continuity claim.
    ///
    /// Exact equality on the closure, and no silent repair. An auto-appended
    /// closing edge is geometry the caller did not draw, and this crate does
    /// not invent geometry — see [`SeamPath::closed`] for the case where you
    /// want that edge and want to say so.
    pub fn new(start: Point2, segments: Vec<EdgeSegment>) -> Result<Self, GeometryError> {
        Self::with_joins(start, segments.into_iter().map(Edge::corner).collect())
    }

    /// [`SeamPath::new`] for a path that makes continuity claims.
    ///
    /// `edges[i].join` describes the join *entering* `edges[i]`, so
    /// `edges[0].join` is the closure join at `start`.
    pub fn with_joins(start: Point2, edges: Vec<Edge>) -> Result<Self, GeometryError> {
        if !start.is_finite() {
            return Err(GeometryError::NonFiniteControlPoint { segment: 0 });
        }
        if edges.is_empty() {
            return Err(GeometryError::TooFewPoints { count: 1 });
        }

        for (index, edge) in edges.iter().enumerate() {
            if edge.geometry().control_points().any(|p| !p.is_finite()) {
                return Err(GeometryError::NonFiniteControlPoint { segment: index });
            }
        }

        let end = edges.last().expect("checked non-empty").end();
        if end != start {
            return Err(GeometryError::PathNotClosed { start, end });
        }

        Ok(Self { start, edges })
    }
```

`closed` needs no edit: it manipulates the caller's `Vec<EdgeSegment>` and still ends by
calling `Self::new(start, segments)`.

Replace the `segments()` accessor at `:226-228`:

```rust
    /// The edges that walk this path back to its start.
    ///
    /// Named `edges`, not `segments`, because it no longer returns segments. A
    /// name that does not describe what it returns is worse than the churn of
    /// renaming it.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }
```

In `flatten` (`:245`) and `max_curvature` (`:318`), change `for segment in &self.segments`
to `for edge in &self.edges`, match on `edge.geometry()` where the segment was matched, and
use `cursor = edge.end()` in place of `cursor = segment.end()`.

Re-export the two new types from `engine/crates/geometry/src/lib.rs:22` — they are public
API and nothing outside the crate can name them otherwise:

```rust
pub use curves::{Edge, EdgeSegment, Join, SeamPath};
```

- [ ] **Step 6: Update the call sites the wire change breaks**

`engine/crates/geometry/tests/curve_oracle.rs:493-498`:

```rust
    assert_eq!(path.edges().len(), open.len() + 1);
    assert_eq!(path.edges().last().unwrap().end(), start);

    // An already-closed path is left exactly as it was.
    let rebuilt: Vec<EdgeSegment> = path.edges().iter().map(|e| e.geometry()).collect();
    let already = SeamPath::closed(start, rebuilt).unwrap();
    assert_eq!(already.edges().len(), path.edges().len());
```

**One more, and it is not a compile error** —
`a_hand_edited_open_path_cannot_be_deserialized` (`curve_oracle.rs:547-555`) carries a
hand-written JSON literal in the *old* wire shape. It compiles fine and fails at runtime
with "missing field `edges`". Its intent — validation is not something a file can skip —
still holds exactly; only the shape it is written in changed:

```rust
    let json = r#"{"start":{"x":0.0,"y":0.0},
                   "edges":[{"geometry":{"kind":"line","to":{"x":10.0,"y":0.0}}}]}"#;
```

Update the fixture, **never the assertion**. If the assertion has to move, that is a
regression wearing a test change as a disguise.

- [ ] **Step 7: Run the full gate**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --workspace --locked'
cmd //c 'C:\Users\User\patal\scripts\cargo.bat fmt --check'
cmd //c 'C:\Users\User\patal\scripts\cargo.bat clippy --workspace --all-targets --locked -- -D warnings'
RUSTDOCFLAGS="-D warnings" cmd //c 'C:\Users\User\patal\scripts\cargo.bat doc --no-deps --workspace --locked'
PATAL_CARGO_DIR='C:\Users\User\patal\apps\desktop\src-tauri' cmd //c 'C:\Users\User\patal\scripts\cargo.bat clippy --all-targets --locked -- -D warnings'
```

Expected: PASS, **138 tests**.

> **Note on the count.** This plan's projections assumed a baseline of 136, which is the
> count on `tile-count-saturates`. That branch is not merged, so a wave branch cut from
> `main` starts at **134** and every projected total in this plan runs two low. Add two
> back once PR #5 lands.

Then confirm the kernel is untouched:

```bash
git diff engine/crates/geometry/src/lib.rs
```

Expected: **exactly one changed line**, the re-export above. No logic, no test, no error
variant — those arrive in Task 2. Anything else is a C12 violation; revert it.

- [ ] **Step 8: Commit**

```bash
git add engine/crates/geometry/src/curves.rs engine/crates/geometry/tests/curve_oracle.rs
git commit -m "Carry the join on the edge, not beside it"
```

---

### Task 2: `Join::Smooth` is validated, not merely recorded (§3.2, validation half)

An earlier draft of the blueprint stored `Smooth` as an unchecked flag, reasoning that
continuity is designer intent rather than geometry. §6's RISK-AGENT pass **vetoed** it: a
`Smooth` claim the coordinates contradict is a plausible-looking wrong value on a path that
feeds the cut line, which is precisely what C1 forbids and precisely the defect class that
got `offset()` fixed last wave. **If a claim cannot be checked, it is not stored.**

This is the one task that modifies `engine/crates/geometry/src/lib.rs`, and only to add
error variants — no logic, no change to any kernel function. C12's stated exception.

**Files:**
- Modify: `engine/crates/geometry/src/lib.rs:28-84` (the `GeometryError` enum and its `Display`)
- Modify: `engine/crates/geometry/src/curves.rs` (constant, tangent helpers, `with_joins`)
- Modify: `engine/crates/geometry/tests/curve_oracle.rs` (new tests)

**Interfaces:**
- Consumes: `Edge`, `Join`, `SeamPath::with_joins` from Task 1.
- Produces: `GeometryError::SmoothJoinUndefinedTangent { join: usize }` and
  `GeometryError::SmoothJoinNotTangent { join: usize, sine: f64, reversed: bool }`;
  `pub const SMOOTH_JOIN_RELATIVE: f64`. `with_joins` keeps its signature and gains the check.

- [ ] **Step 1: Write the failing tests**

Append to `engine/crates/geometry/tests/curve_oracle.rs`:

```rust
/// A square with a genuinely smooth join is impossible — every corner turns
/// 90°. This helper builds a shape whose first join *is* smooth: a straight
/// run into a cubic whose first handle continues the same direction.
fn line_into_collinear_cubic() -> (Point2, Vec<Edge>) {
    let start = Point2::new(0.0, 0.0);
    // Edge 0: a line east to (10, 0). Its end tangent is +x.
    // Edge 1: a cubic from (10, 0) whose c1 is (20, 0) — start tangent +x.
    // The join *entering* edge 1 is therefore smooth.
    let edges = vec![
        Edge::corner(EdgeSegment::Line { to: Point2::new(10.0, 0.0) }),
        Edge::new(
            EdgeSegment::Cubic {
                c1: Point2::new(20.0, 0.0),
                c2: Point2::new(30.0, 10.0),
                to: Point2::new(30.0, 20.0),
            },
            Join::Smooth,
        ),
        Edge::corner(EdgeSegment::Line { to: start }),
    ];
    (start, edges)
}

#[test]
fn a_line_meeting_a_cubic_in_line_is_a_legal_smooth_join() {
    // Ordinary pattern making: a straight hem meeting a curved side seam.
    let (start, edges) = line_into_collinear_cubic();
    let path = SeamPath::with_joins(start, edges).expect("collinear handles are smooth");
    assert_eq!(path.edges()[1].join(), Join::Smooth);
}

#[test]
fn a_smooth_claim_the_coordinates_contradict_is_refused() {
    let start = Point2::new(0.0, 0.0);
    // Edge 0 ends heading +x; edge 1's c1 heads +y. That is a visible corner,
    // and calling it smooth is a lie the cut line would inherit.
    let edges = vec![
        Edge::corner(EdgeSegment::Line { to: Point2::new(10.0, 0.0) }),
        Edge::new(
            EdgeSegment::Cubic {
                c1: Point2::new(10.0, 10.0),
                c2: Point2::new(20.0, 20.0),
                to: Point2::new(30.0, 20.0),
            },
            Join::Smooth,
        ),
        Edge::corner(EdgeSegment::Line { to: start }),
    ];

    let err = SeamPath::with_joins(start, edges).expect_err("a 90 degree turn is not smooth");
    match err {
        GeometryError::SmoothJoinNotTangent { join, sine, reversed } => {
            assert_eq!(join, 1);
            assert!(sine > 0.5, "a right angle has sine 1, got {sine}");
            assert!(!reversed);
        }
        other => panic!("wrong error: {other:?}"),
    }
}

#[test]
fn a_smooth_claim_across_a_reversal_is_refused_even_though_it_is_collinear() {
    let start = Point2::new(0.0, 0.0);
    // Edge 0 ends heading +x. Edge 1's c1 heads -x: parallel, opposite sign.
    // The sine is ~0, so only the direction check catches this one.
    let edges = vec![
        Edge::corner(EdgeSegment::Line { to: Point2::new(10.0, 0.0) }),
        Edge::new(
            EdgeSegment::Cubic {
                c1: Point2::new(5.0, 0.0),
                c2: Point2::new(20.0, 10.0),
                to: Point2::new(30.0, 20.0),
            },
            Join::Smooth,
        ),
        Edge::corner(EdgeSegment::Line { to: start }),
    ];

    let err = SeamPath::with_joins(start, edges).expect_err("a cusp is not smooth");
    match err {
        GeometryError::SmoothJoinNotTangent { join, reversed, .. } => {
            assert_eq!(join, 1);
            assert!(reversed, "the tangents are anti-parallel");
        }
        other => panic!("wrong error: {other:?}"),
    }
}

#[test]
fn a_degenerate_handle_with_a_smooth_claim_errors_rather_than_becoming_a_corner() {
    let start = Point2::new(0.0, 0.0);
    // c1 sits exactly on the cubic's own start, so the outgoing tangent is
    // the zero vector and "smooth" has no meaning. Silently treating that as
    // a corner would store a claim nobody checked.
    let edges = vec![
        Edge::corner(EdgeSegment::Line { to: Point2::new(10.0, 0.0) }),
        Edge::new(
            EdgeSegment::Cubic {
                c1: Point2::new(10.0, 0.0),
                c2: Point2::new(20.0, 10.0),
                to: Point2::new(30.0, 20.0),
            },
            Join::Smooth,
        ),
        Edge::corner(EdgeSegment::Line { to: start }),
    ];

    let err = SeamPath::with_joins(start, edges).expect_err("no tangent, no claim");
    assert!(matches!(
        err,
        GeometryError::SmoothJoinUndefinedTangent { join: 1 }
    ));
}

#[test]
fn a_corner_claim_is_never_checked_however_sharp_it_is() {
    // The absence of a claim cannot be false. A square is all corners and
    // must construct without any tangent arithmetic running against it.
    let start = Point2::new(0.0, 0.0);
    SeamPath::new(
        start,
        vec![
            EdgeSegment::Line { to: Point2::new(10.0, 0.0) },
            EdgeSegment::Line { to: Point2::new(10.0, 10.0) },
            EdgeSegment::Line { to: Point2::new(0.0, 10.0) },
            EdgeSegment::Line { to: start },
        ],
    )
    .expect("a square is four legal corners");
}

#[test]
fn a_falsely_smooth_path_cannot_be_smuggled_in_through_a_file() {
    // C6: serde routes through the validating constructor, so a hand-edited
    // document gets the same refusal an API caller gets.
    let json = r#"{
        "start": {"x": 0.0, "y": 0.0},
        "edges": [
            {"geometry": {"kind": "line", "to": {"x": 10.0, "y": 0.0}}, "join": "corner"},
            {"geometry": {"kind": "line", "to": {"x": 10.0, "y": 10.0}}, "join": "smooth"},
            {"geometry": {"kind": "line", "to": {"x": 0.0, "y": 0.0}}, "join": "corner"}
        ]
    }"#;
    let err = serde_json::from_str::<SeamPath>(json).expect_err("a right angle is not smooth");
    assert!(err.to_string().contains("smooth"), "{err}");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-geometry --test curve_oracle'
```

Expected: FAIL to compile — "no variant named `SmoothJoinNotTangent`".

- [ ] **Step 3: Add the two error variants**

In `engine/crates/geometry/src/lib.rs`, after `ToleranceNotPositive` (`:82`), add:

```rust
    /// A `Join::Smooth` sits where a tangent does not exist — a cubic whose
    /// first handle is coincident with its own start, or a zero-length edge.
    ///
    /// Refused rather than quietly demoted to a corner: a demotion stores a
    /// claim nobody checked, and this crate does not do that.
    SmoothJoinUndefinedTangent { join: usize },
    /// A `Join::Smooth` that the coordinates on either side contradict.
    ///
    /// `sine` is the sine of the angle between the tangents, so it is
    /// scale-free and reads directly as "how far off tangency this is".
    /// `reversed` distinguishes the cusp case — tangents parallel but
    /// pointing opposite ways — where `sine` is near zero and only the
    /// direction check catches it.
    SmoothJoinNotTangent { join: usize, sine: f64, reversed: bool },
```

Extend the `Display` impl in the same file with matching arms:

```rust
            Self::SmoothJoinUndefinedTangent { join } => write!(
                f,
                "the smooth join entering edge {join} has no defined tangent: a control \
                 point is coincident with the point it leaves, so there is no direction \
                 to be smooth about"
            ),
            Self::SmoothJoinNotTangent { join, sine, reversed } => {
                if *reversed {
                    write!(
                        f,
                        "the smooth join entering edge {join} reverses direction: the \
                         tangents are parallel but point opposite ways, which is a cusp, \
                         not a smooth join"
                    )
                } else {
                    write!(
                        f,
                        "the smooth join entering edge {join} is not tangent: the angle \
                         between the tangents has sine {sine}, above the {SMOOTH_JOIN_RELATIVE} \
                         this crate accepts as float noise"
                    )
                }
            }
```

Import `SMOOTH_JOIN_RELATIVE` from `crate::curves` at the top of `lib.rs`, or inline the
literal in the message if that import would create a cycle in the module's existing
ordering — the value matters, the provenance does not.

**Do not touch anything else in this file.** Re-run `git diff engine/crates/geometry/src/lib.rs`
at the end of the task and confirm the only hunks are the enum and the `Display` arms.

- [ ] **Step 4: Add the constant and the tangent helpers**

In `engine/crates/geometry/src/curves.rs`, after `CLOSURE_SNAP_RELATIVE` (`:65`):

```rust
/// How far from parallel two tangents may sit and still be called smooth.
///
/// Compared against the *sine* of the angle between them, which is the cross
/// product divided by both magnitudes — so this is scale-free and means the
/// same thing on a 10mm buttonhole and a 2000mm bolt of cloth, exactly as
/// [`CLOSURE_SNAP_RELATIVE`] does for distance.
///
/// At 1e-9 radians, a 100mm handle deviates by 1e-7mm: ten thousand times
/// finer than a micron, far below any coordinate a designer expresses, and
/// several orders above the ~1e-13 relative noise a cross product of
/// millimetre-scale coordinates leaves behind. A designer's tool that draws a
/// smooth join computes collinear handles exactly; this threshold exists for
/// the float noise on the way to and from a file, not for hand-typed
/// coordinates.
///
/// If it ever proves too tight, widen it and pin the new value in a named test
/// with the reasoning. Never drop the check to make a case pass.
pub const SMOOTH_JOIN_RELATIVE: f64 = 1.0e-9;
```

At the bottom of `curves.rs`, beside the other free functions:

```rust
/// The direction a segment leaves `from` in, or `None` where that direction
/// is undefined.
fn start_tangent(segment: EdgeSegment, from: Point2) -> Option<(f64, f64)> {
    let head = match segment {
        EdgeSegment::Line { to } => to,
        EdgeSegment::Cubic { c1, .. } => c1,
    };
    non_degenerate(head.x - from.x, head.y - from.y)
}

/// The direction a segment arrives at its end in, or `None` where that
/// direction is undefined.
fn end_tangent(segment: EdgeSegment, from: Point2) -> Option<(f64, f64)> {
    let (tail, head) = match segment {
        EdgeSegment::Line { to } => (from, to),
        EdgeSegment::Cubic { c2, to, .. } => (c2, to),
    };
    non_degenerate(head.x - tail.x, head.y - tail.y)
}

/// A tangent vector, unless it has collapsed to a point.
fn non_degenerate(dx: f64, dy: f64) -> Option<(f64, f64)> {
    let length = dx.hypot(dy);
    (length > 0.0 && length.is_finite()).then_some((dx, dy))
}
```

- [ ] **Step 5: Wire the check into `with_joins`**

Insert into `SeamPath::with_joins`, after the `PathNotClosed` check and before
`Ok(Self { .. })`:

```rust
        // Where each edge begins. Edge 0 begins at `start`; every other edge
        // begins where its predecessor ended.
        let mut origins: Vec<Point2> = Vec::with_capacity(edges.len());
        let mut cursor = start;
        for edge in &edges {
            origins.push(cursor);
            cursor = edge.end();
        }

        let count = edges.len();
        for index in 0..count {
            if edges[index].join() != Join::Smooth {
                continue;
            }

            // `edges[i].join` describes the join *entering* edge i, so
            // edges[0]'s join is the closure join and its predecessor is the
            // last edge.
            let previous = (index + count - 1) % count;
            let incoming = end_tangent(edges[previous].geometry(), origins[previous]);
            let outgoing = start_tangent(edges[index].geometry(), origins[index]);

            let (Some((ix, iy)), Some((ox, oy))) = (incoming, outgoing) else {
                return Err(GeometryError::SmoothJoinUndefinedTangent { join: index });
            };

            let cross = ix * oy - iy * ox;
            let dot = ix * ox + iy * oy;
            let sine = cross.abs() / (ix.hypot(iy) * ox.hypot(oy));

            // `reversed` means a cusp — parallel but pointing opposite ways —
            // so it is `dot < 0`, not `dot <= 0`. See the correction below.
            if sine > SMOOTH_JOIN_RELATIVE || dot < 0.0 {
                return Err(GeometryError::SmoothJoinNotTangent {
                    join: index,
                    sine,
                    reversed: dot < 0.0,
                });
            }
        }
```

Note the loop `continue`s on `Corner` before any arithmetic runs. A corner claim cannot be
false, so it is never checked however sharp the angle is — which is what the square test in
Step 1 pins.

**CORRECTION, made while executing (2026-08-16).** This snippet originally read
`dot <= 0.0` in both the condition and the `reversed` field. That contradicts Step 1's own
`a_smooth_claim_the_coordinates_contradict_is_refused`, which builds a perpendicular join
and asserts `!reversed`. A perpendicular join has `dot == 0.0` exactly, so the original
snippet labelled a right angle a cusp and that test failed. The test is right; the snippet
was wrong. The strict comparison loses nothing: `dot == 0` with two non-zero tangents
implies `sine == 1`, which always exceeds the threshold, so the join is still refused — by
the sine branch, where it belongs. The two rejection reasons are now disjoint rather than
overlapping.

- [ ] **Step 6: Run the tests**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --workspace --locked'
```

Expected: PASS, **144 tests** (138 from Task 1 + 6 new). The plan originally projected 145
from a 139-test Task 1; see the corrected baseline note under "Baseline at plan time".

- [ ] **Step 7: Verify the container earned its keep**

Not a test — a five-minute review check, and the only evidence revision 6 bought what it
claimed. Confirm by inspection that adding a second field to `Edge` (say
`allowance_mm: Option<f64>`) would require touching **no** invariant in `with_joins` and
**no** length check anywhere. Write the answer into the commit message. If it would require
touching an invariant, the container is not doing its job and that is worth knowing now.

- [ ] **Step 8: Commit**

```bash
git add engine/crates/geometry/src/lib.rs engine/crates/geometry/src/curves.rs \
        engine/crates/geometry/tests/curve_oracle.rs
git commit -m "Check a smooth join against the coordinates that claim it"
```

---

### Task 3: `PatternBoundary` → `SeamPath` lift (§3.1)

The conversion every other task needs: the migration needs it, the piece's convenience
constructor needs it, and Swift's model needs the concept. Written against the final `Edge`
shape because Task 1 already landed it — see **R4**.

**Files:**
- Modify: `engine/crates/geometry/src/curves.rs` (add `SeamPath::from_boundary`)
- Modify: `engine/crates/geometry/tests/properties.rs` (the headline property)

**Interfaces:**
- Consumes: `Edge::corner`, the `edges` field, from Task 1.
- Produces: `SeamPath::from_boundary(&PatternBoundary) -> SeamPath`. **Infallible** — returns
  `Self`, not `Result`.

- [ ] **Step 1: Write the failing property**

Append to `engine/crates/geometry/tests/properties.rs`:

```rust
proptest! {
    /// The headline property of the whole wave.
    ///
    /// Bit-identical, not within-epsilon. The lift performs no float
    /// arithmetic — it moves existing coordinates into a new container — so
    /// flattening it back must reproduce the input exactly, at every
    /// tolerance. If this ever needs an epsilon, the lift has acquired
    /// arithmetic that does not belong in it.
    #[test]
    fn lifting_a_boundary_and_flattening_it_back_is_bit_identical(
        boundary in any_boundary(),
        tolerance in 1.0e-6f64..100.0f64,
    ) {
        let lifted = SeamPath::from_boundary(&boundary);
        let flattened = lifted.flatten(tolerance).expect("a lifted polygon always flattens");
        prop_assert_eq!(flattened.points(), boundary.points());
    }

    /// A lifted polygon is all corners. It has to be: the lift cannot know
    /// intent the polygon never carried, and inventing a `Smooth` claim would
    /// be inventing exactly the kind of unverifiable assertion C1 forbids.
    #[test]
    fn a_lifted_boundary_claims_no_continuity(boundary in any_boundary()) {
        let lifted = SeamPath::from_boundary(&boundary);
        prop_assert!(lifted.edges().iter().all(|e| e.join() == Join::Corner));
        prop_assert_eq!(lifted.edges().len(), boundary.points().len());
    }
}
```

If `any_boundary()` does not already exist in that file, reuse whatever generator the
existing properties use for a valid `PatternBoundary` and name it in the two tests above
rather than writing a second generator.

**AS EXECUTED (2026-08-16).** `any_boundary()` does not exist. The generator is
`convex_boundary()`, which yields `Option<PatternBoundary>` and needs the
`let Some(boundary) = boundary else { return Ok(()) };` idiom the file already uses.

**A third property was added, and it matters.** `convex_boundary()` generates only convex
shapes — it exists because `offset` needs inputs whose answer is guaranteed. The lift has
no such requirement, and S4 claims bit-identity for *every* valid boundary, so
convex-only would have under-tested the headline claim of the wave.
`the_lift_is_bit_identical_on_boundaries_that_are_not_convex` runs the same assertion over
the existing `point_soup()` generator, filtered through `PatternBoundary::new(..).ok()`,
which covers concave and self-crossing boundaries. Still no new generator.

- [ ] **Step 2: Run the property to verify it fails**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-geometry --test properties'
```

Expected: FAIL to compile — "no function or associated item named `from_boundary`".

- [ ] **Step 3: Write the lift**

Add to the `impl SeamPath` block in `engine/crates/geometry/src/curves.rs`, directly after
`closed`:

```rust
    /// Lifts a polygon into an authored path of straight edges.
    ///
    /// # Why this does not invent geometry
    ///
    /// Appending the closing edge back to `start` looks like exactly what C9
    /// forbids, and it is worth being explicit about why it is not. A
    /// [`PatternBoundary`] is *defined* as a closed polygon — the edge from
    /// its last point back to its first already exists, and
    /// [`PatternBoundary::perimeter`] has always counted it. This makes that
    /// edge explicit; it does not create one.
    ///
    /// That is different in kind from [`SeamPath::closed`], which spans a gap
    /// the designer actually left, and which is why *that* function requires
    /// the caller to opt in and this one does not.
    ///
    /// # Why this is infallible
    ///
    /// A valid `PatternBoundary` already guarantees at least three finite,
    /// deduplicated points, which is strictly more than [`SeamPath::new`]
    /// demands. There is no failure to report, and a `Result` here would be a
    /// lie about what can go wrong.
    ///
    /// No float arithmetic happens anywhere in this function. That is what
    /// makes the round-trip property in `tests/properties.rs` bit-exact
    /// rather than approximate.
    pub fn from_boundary(boundary: &PatternBoundary) -> Self {
        let points = boundary.points();
        let start = points[0];
        let edges = points[1..]
            .iter()
            .copied()
            .chain(std::iter::once(start))
            .map(|to| Edge::corner(EdgeSegment::Line { to }))
            .collect();

        Self { start, edges }
    }
```

Indexing `points[0]` and slicing `points[1..]` are safe without a check for the reason the
doc comment gives: `PatternBoundary::new` rejects anything under three points with
`TooFewPoints`, so a `&PatternBoundary` that exists has at least three.

- [ ] **Step 4: Run the property**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-geometry --test properties'
```

Expected: PASS. If proptest shrinks a counterexample, **do not add an epsilon**. Promote the
shrunk case into a named test, then find the arithmetic that crept into the lift and remove
it — §5.1's stated fallback is "none needed", and that is deliberate.

- [ ] **Step 5: Regenerate the regressions file if proptest wrote to it**

```bash
git status --short engine/crates/geometry/tests/properties.proptest-regressions
```

A modified regressions file is expected noise once generators change, not a defect — but it
will look like one in review, so mention it in the commit message if it moved.

- [ ] **Step 6: Run the full gate**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --workspace --locked'
```

Expected: PASS, **147 tests** (144 from Task 2 + 3 new properties). This number is right
even though Task 2's was not — the extra property absorbs the one-test baseline drift, and
every count from here to Task 6 lands exactly as projected.

- [ ] **Step 7: Commit**

```bash
git add engine/crates/geometry/src/curves.rs engine/crates/geometry/tests/properties.rs \
        engine/crates/geometry/tests/properties.proptest-regressions
git commit -m "Lift a polygon into the path it always implied"
```

---

### Task 4: `GrainLine` (§3.3)

Not speculative. DXF-AAMA/ASTM defines grain line as a specific entity and export is
already built, so this is the field export will need — added while the schema is moving
anyway.

Lives in `patal-pattern`, not `patal-geometry`: it is an attribute of a piece, and the
geometry crate stays kernel-plus-curves. It gets its own file because `pattern/src/lib.rs`
is already 729 lines and this wave adds substantially to it.

**Files:**
- Create: `engine/crates/pattern/src/grain.rs`
- Modify: `engine/crates/pattern/src/lib.rs` (add `mod grain;`, re-export, add the error variant)

**Interfaces:**
- Consumes: `patal_geometry::Point2`.
- Produces: `pub struct GrainLine` with
  `GrainLine::new(angle_deg: f64, anchor: Point2) -> Result<GrainLine, PatternError>`,
  `GrainLine::angle_deg(&self) -> f64`, `GrainLine::anchor(&self) -> Point2`;
  `PatternError::InvalidGrainLine { field: &'static str, value: f64 }`.

- [ ] **Step 1: Write the failing tests**

Create `engine/crates/pattern/src/grain.rs` with only its test module to begin with:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_angle_past_a_full_turn_comes_back_inside_one() {
        let grain = GrainLine::new(450.0, Point2::new(0.0, 0.0)).expect("valid");
        assert_eq!(grain.angle_deg(), 90.0);
    }

    #[test]
    fn a_negative_angle_normalises_forward_not_backward() {
        let grain = GrainLine::new(-170.0, Point2::new(0.0, 0.0)).expect("valid");
        assert_eq!(grain.angle_deg(), 190.0);
    }

    #[test]
    fn one_hundred_and_ninety_degrees_stays_one_hundred_and_ninety() {
        // The whole reason normalisation is [0,360) and not [0,180). A grain
        // line is directional, not axial: napped fabrics — velvet, corduroy —
        // require every piece laid the same way up, and folding 190 onto 10
        // would silently destroy that. It looks like a tidy simplification
        // right up until someone cuts a velvet jacket.
        let grain = GrainLine::new(190.0, Point2::new(0.0, 0.0)).expect("valid");
        assert_eq!(grain.angle_deg(), 190.0);
    }

    #[test]
    fn a_full_turn_is_zero_not_three_hundred_and_sixty() {
        let grain = GrainLine::new(360.0, Point2::new(0.0, 0.0)).expect("valid");
        assert_eq!(grain.angle_deg(), 0.0);
    }

    #[test]
    fn a_non_finite_angle_is_refused_rather_than_normalised_into_something_plausible() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let err = GrainLine::new(bad, Point2::new(0.0, 0.0)).expect_err("refused");
            assert!(matches!(
                err,
                PatternError::InvalidGrainLine { field: "angle_deg", .. }
            ));
        }
    }

    #[test]
    fn a_non_finite_anchor_is_refused() {
        let err = GrainLine::new(0.0, Point2::new(f64::NAN, 0.0)).expect_err("refused");
        assert!(matches!(
            err,
            PatternError::InvalidGrainLine { field: "anchor", .. }
        ));
    }

    #[test]
    fn a_hand_edited_grain_line_cannot_skip_the_check() {
        // C6: serde routes through the constructor.
        let json = r#"{"angle_deg": 1e400, "anchor": {"x": 0.0, "y": 0.0}}"#;
        assert!(serde_json::from_str::<GrainLine>(json).is_err());
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-pattern grain'
```

Expected: FAIL to compile — `grain.rs` is not a module yet and `GrainLine` does not exist.

- [ ] **Step 3: Implement**

Prepend to `engine/crates/pattern/src/grain.rs`:

```rust
//! The direction a piece is laid on the cloth.

use patal_geometry::Point2;
use serde::{Deserialize, Serialize};

use crate::PatternError;

/// The wire shape of a [`GrainLine`], so serde routes through the validator.
#[derive(Serialize, Deserialize)]
struct GrainLineData {
    angle_deg: f64,
    anchor: Point2,
}

/// Which way the warp runs through a piece, and where the marking sits.
///
/// The angle is normalised into `[0, 360)` — a full circle, not a half one.
/// Grain is **directional, not axial**: napped fabrics such as velvet and
/// corduroy shade differently depending on which way the pile lies, so every
/// piece must be laid the same way up. Folding 190° onto 10° would lose
/// exactly the constraint that makes such a garment cuttable.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "GrainLineData", into = "GrainLineData")]
pub struct GrainLine {
    angle_deg: f64,
    anchor: Point2,
}

impl GrainLine {
    pub fn new(angle_deg: f64, anchor: Point2) -> Result<Self, PatternError> {
        if !angle_deg.is_finite() {
            return Err(PatternError::InvalidGrainLine {
                field: "angle_deg",
                value: angle_deg,
            });
        }
        if !anchor.is_finite() {
            // Report whichever coordinate is at fault, so the message names a
            // number the caller can find in their file.
            let value = if anchor.x.is_finite() { anchor.y } else { anchor.x };
            return Err(PatternError::InvalidGrainLine { field: "anchor", value });
        }

        // `rem_euclid` lands in [0, 360) for negative input too, which is the
        // whole reason it is used here rather than `%`.
        Ok(Self { angle_deg: angle_deg.rem_euclid(360.0), anchor })
    }

    pub fn angle_deg(&self) -> f64 {
        self.angle_deg
    }

    pub fn anchor(&self) -> Point2 {
        self.anchor
    }
}

impl From<GrainLine> for GrainLineData {
    fn from(grain: GrainLine) -> Self {
        Self { angle_deg: grain.angle_deg, anchor: grain.anchor }
    }
}

impl TryFrom<GrainLineData> for GrainLine {
    type Error = PatternError;

    fn try_from(data: GrainLineData) -> Result<Self, Self::Error> {
        Self::new(data.angle_deg, data.anchor)
    }
}
```

In `engine/crates/pattern/src/lib.rs`, add `mod grain;` and `pub use grain::GrainLine;`
near the top, and add the error variant to `PatternError` plus its `Display` arm:

```rust
    /// A grain line whose angle or anchor is not a usable number.
    InvalidGrainLine { field: &'static str, value: f64 },
```

```rust
            Self::InvalidGrainLine { field, value } => write!(
                f,
                "grain line {field} is {value}, which is not a finite number"
            ),
```

Add `InvalidGrainLine { .. }` to the `None` arm of the `source()` match so the exhaustive
list stays exhaustive.

- [ ] **Step 4: Run the tests**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --workspace --locked'
```

Expected: PASS, **154 tests** (147 + 7 new).

- [ ] **Step 5: Commit**

```bash
git add engine/crates/pattern/src/grain.rs engine/crates/pattern/src/lib.rs
git commit -m "Give a piece a grain line, and make it directional"
```

---

### Task 5: `PieceId` and `find_piece_by_id` (§3.4)

ADR-004 records this as the remaining open identity divergence: Swift's `PatternPiece`
carries a `UUID`, Rust's has no identity field at all, which is why the piece's document
shape is still Swift-to-Swift only. A field with no lookup is bytes on disk, so the reader
ships in the same task — grading and export both index pieces by identity.

**Files:**
- Modify: `engine/crates/pattern/Cargo.toml` (**R3** — add `uuid`)
- Modify: `engine/crates/pattern/src/lib.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `pub struct PieceId` with `PieceId::new() -> PieceId`,
  `PieceId::as_uuid(&self) -> Uuid`, `Display`;
  `PatternPiece::id(&self) -> PieceId`;
  `Project::find_piece_by_id(&self, id: PieceId) -> Option<&PatternPiece>`.

- [ ] **Step 1: Write the failing tests**

Add to the `mod tests` block in `engine/crates/pattern/src/lib.rs`:

```rust
    #[test]
    fn two_pieces_never_share_an_id() {
        let a = PatternPiece::new("Front", square_boundary(100.0));
        let b = PatternPiece::new("Front", square_boundary(100.0));
        assert_ne!(a.id(), b.id(), "same name, different identity");
    }

    #[test]
    fn a_piece_is_findable_by_id_as_well_as_by_name() {
        let mut project = Project::new("Blouse");
        let piece = PatternPiece::new("Front", square_boundary(100.0));
        let id = piece.id();
        project.add_piece(piece);

        assert_eq!(project.find_piece_by_id(id).map(|p| p.name.as_str()), Some("Front"));
        assert!(project.find_piece_by_id(PieceId::new()).is_none());
    }

    #[test]
    fn an_id_survives_a_round_trip_and_is_a_plain_string_on_the_wire() {
        // A plain string, not a nested object, so Swift's Foundation.UUID
        // decodes it directly — the same treatment MaterialId got.
        let piece = PatternPiece::new("Front", square_boundary(100.0));
        let json = serde_json::to_value(&piece).expect("serializes");
        assert!(json["id"].is_string(), "id must be a bare string, got {}", json["id"]);

        let restored: PatternPiece = serde_json::from_value(json).expect("round trips");
        assert_eq!(restored.id(), piece.id());
    }
```

- [ ] **Step 2: Run to verify it fails**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-pattern'
```

Expected: FAIL to compile — "cannot find type `PieceId`".

- [ ] **Step 3: Add the dependency**

In `engine/crates/pattern/Cargo.toml`, under `[dependencies]`:

```toml
uuid = { workspace = true }
```

The workspace already pins `uuid = { version = "1", features = ["v4", "serde"] }`, so no
version decision is being made here.

- [ ] **Step 4: Implement**

In `engine/crates/pattern/src/lib.rs`, add `use uuid::Uuid;` and, beside the other types:

```rust
/// A piece's stable identity, independent of its name.
///
/// Copied deliberately from `MaterialId` rather than invented: same UUID
/// backing, same `serde(transparent)` so it is a plain string on the wire and
/// `Foundation.UUID` reads it directly, same refusal to implement `Default`.
///
/// Names are not identity. Two pieces can legitimately be called "Front", and
/// grading and export both need to say *which* piece without depending on a
/// string the designer may rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PieceId(Uuid);

impl PieceId {
    /// Mints a fresh identity. Deliberately not `Default`: an id should be
    /// created where a piece is created, never conjured to fill a gap.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for PieceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

Add `id: PieceId` as a **private** field on `PatternPiece` and on `PatternPieceData`, set it
in `PatternPiece::new` with `PieceId::new()`, carry it through both conversion impls, and
add the accessor:

```rust
    /// This piece's identity. No setter: an id describes which piece this is,
    /// and letting a caller assign one would let two pieces claim to be the
    /// same piece.
    pub fn id(&self) -> PieceId {
        self.id
    }
```

Add the reader to `impl Project`, beside `find_piece`:

```rust
    pub fn find_piece_by_id(&self, id: PieceId) -> Option<&PatternPiece> {
        self.pieces.iter().find(|p| p.id == id)
    }
```

Keep `find_piece(&str)` exactly as it is. It has callers and names stay useful.

- [ ] **Step 5: Run and commit**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --workspace --locked'
```

Expected: PASS, **157 tests**. Note that `PatternPieceData` now round-trips an `id`, so any
existing test asserting an exact serialized shape for a piece needs its expectation updated
— update the expectation, never the assertion's intent.

**AS EXECUTED (2026-08-16), two things this step does not tell you:**

1. **`--locked` fails before the lockfiles move,** with "cannot update the lock file …
   because --locked was passed". Adding a dependency means running
   `cargo check --workspace` once first. Do not reach for `--offline` or drop `--locked`;
   the lock is meant to move here.
2. **There are two lockfiles, not one.** `apps/desktop/src-tauri` is its own workspace with
   its own `Cargo.lock`, so the fifth gate fails identically until it is updated the same
   way. The `git add` below covers both.
3. **The test that breaks is named.** `deserializing_negative_seam_allowance_is_rejected`
   carries a JSON literal in the old wire shape; it now fails on the missing `id` before it
   ever reaches the seam-allowance validator. Fix the fixture by adding an `"id"` key with
   any valid UUID string. Its intent — a hand-edited file cannot skip validation — holds
   exactly as written, so the assertion does not move. Same failure class as Task 1's
   `a_hand_edited_open_path_cannot_be_deserialized`.

**One design point the plan left implicit:** `id` is a *required* field on the wire, not
`#[serde(default)]`. That is the opposite of `join`'s treatment and the asymmetry is
deliberate — `Corner` is the absence of a claim so defaulting it invents nothing, whereas
minting an id for a v2 file that omitted one would give that piece a different identity on
every load. v1 files have no ids, but that is the migration's job (Task 8), not serde's.

```bash
git add engine/crates/pattern/Cargo.toml engine/crates/pattern/src/lib.rs \
        engine/Cargo.lock apps/desktop/src-tauri/Cargo.lock
git commit -m "Give a piece an identity its name cannot carry"
```

---

### Task 6: Project flatten tolerance, and the `Default` trap (§3.5)

The moment a piece holds curves, `cut_boundary()` needs a tolerance, and it has to survive
the file — otherwise a reload silently produces a different cut line, which C1 forbids.

**The single most likely silent break in this wave lives here.** `Project` and `ProjectData`
both `#[derive(Default)]` today (`pattern/src/lib.rs:256,272`). Adding a validated
tolerance makes the derived impl produce `0.0`, which the setter rejects — so
`Project::default()` would mint an invalid project through a path that never calls the
validator. It fires during deserialization too, because `ProjectData.materials` carries
`#[serde(default)]`.

**Files:**
- Modify: `engine/crates/pattern/src/lib.rs:256-304`

**Interfaces:**
- Consumes: nothing.
- Produces: `pub const DEFAULT_FLATTEN_TOLERANCE_MM: f64 = 0.01`;
  `Project::flatten_tolerance_mm(&self) -> f64`;
  `Project::set_flatten_tolerance_mm(&mut self, f64) -> Result<(), PatternError>`.

- [ ] **Step 1: Write the failing tests**

```rust
    #[test]
    fn a_default_project_carries_a_usable_tolerance() {
        // The trap this test exists for: a derived Default would produce 0.0,
        // which set_flatten_tolerance_mm refuses — so Project::default()
        // would mint a project the validator would have rejected, through a
        // path that never runs it.
        let project = Project::default();
        assert_eq!(project.flatten_tolerance_mm(), DEFAULT_FLATTEN_TOLERANCE_MM);
        assert!(project.flatten_tolerance_mm() > 0.0);
    }

    #[test]
    fn a_project_deserialized_without_a_tolerance_key_still_gets_a_valid_one() {
        // Fires through ProjectData's serde defaults, one level below the
        // obvious call site.
        let json = r#"{"name": "Blouse", "pieces": [], "measurements": []}"#;
        let project: Project = serde_json::from_str(json).expect("loads");
        assert_eq!(project.flatten_tolerance_mm(), DEFAULT_FLATTEN_TOLERANCE_MM);
    }

    #[test]
    fn a_tolerance_that_cannot_describe_a_curve_is_refused() {
        let mut project = Project::new("Blouse");
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(project.set_flatten_tolerance_mm(bad).is_err(), "{bad} accepted");
        }
        assert_eq!(project.flatten_tolerance_mm(), DEFAULT_FLATTEN_TOLERANCE_MM);
    }

    #[test]
    fn a_tolerance_survives_the_file() {
        let mut project = Project::new("Blouse");
        project.set_flatten_tolerance_mm(0.05).expect("valid");
        let json = serde_json::to_string(&project).expect("serializes");
        let restored: Project = serde_json::from_str(&json).expect("loads");
        assert_eq!(restored.flatten_tolerance_mm(), 0.05);
    }
```

- [ ] **Step 2: Run to verify it fails**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-pattern'
```

Expected: FAIL to compile — "no method named `flatten_tolerance_mm`".

- [ ] **Step 3: Implement**

```rust
/// The flattening tolerance a new project starts with, in millimetres.
///
/// 0.01mm against ADR-003's 0.4mm industrial-cutter figure: forty times finer
/// than any cutter can execute and far finer than cloth can hold. The last
/// wave measured this exact tolerance at roughly 1% of a 120Hz frame for one
/// piece's full drag path, so it is affordable on evidence rather than on
/// assertion.
///
/// There is deliberately **no upper bound**. A tolerance of 1e9 turns every
/// curve into a straight line, which is useless but not *wrong* in the
/// correct-or-loud sense, and inventing a ceiling means inventing a number.
/// Revisit if a real user ever sets one.
pub const DEFAULT_FLATTEN_TOLERANCE_MM: f64 = 0.01;
```

Remove `Default` from the derive list on **both** `Project` and `ProjectData`, add
`flatten_tolerance_mm: f64` to both, and hand-write the impls:

```rust
impl Default for Project {
    fn default() -> Self {
        Self {
            name: String::new(),
            pieces: Vec::new(),
            measurements: Vec::new(),
            materials: MaterialLibrary::default(),
            flatten_tolerance_mm: DEFAULT_FLATTEN_TOLERANCE_MM,
        }
    }
}

impl Default for ProjectData {
    fn default() -> Self {
        Self {
            name: String::new(),
            pieces: Vec::new(),
            measurements: Vec::new(),
            materials: MaterialLibrary::default(),
            flatten_tolerance_mm: DEFAULT_FLATTEN_TOLERANCE_MM,
        }
    }
}
```

On `ProjectData`'s new field:

```rust
    #[serde(default = "default_flatten_tolerance")]
    flatten_tolerance_mm: f64,
```

```rust
fn default_flatten_tolerance() -> f64 {
    DEFAULT_FLATTEN_TOLERANCE_MM
}
```

Carry the field through both conversion impls, and validate it in
`TryFrom<ProjectData> for Project` — a file is exactly where an invalid tolerance would
arrive from:

```rust
        let mut project = Project {
            name: data.name,
            pieces: data.pieces,
            measurements: data.measurements,
            materials: data.materials,
            flatten_tolerance_mm: DEFAULT_FLATTEN_TOLERANCE_MM,
        };
        project.set_flatten_tolerance_mm(data.flatten_tolerance_mm)?;
        project.check_material_references()?;
        Ok(project)
```

Accessor and validating setter:

```rust
    pub fn flatten_tolerance_mm(&self) -> f64 {
        self.flatten_tolerance_mm
    }

    /// Sets the flattening tolerance, refusing values that cannot describe a
    /// curve. Reuses the geometry crate's own error so one condition has one
    /// name across both layers.
    pub fn set_flatten_tolerance_mm(&mut self, value_mm: f64) -> Result<(), PatternError> {
        if !value_mm.is_finite() || value_mm <= 0.0 {
            return Err(PatternError::Geometry(GeometryError::ToleranceNotPositive {
                tolerance_mm: value_mm,
            }));
        }
        self.flatten_tolerance_mm = value_mm;
        Ok(())
    }
```

Confirm `Project::new` sets the default rather than relying on a derive.

- [ ] **Step 4: Run and commit**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --workspace --locked'
```

Expected: PASS, **161 tests**.

```bash
git add engine/crates/pattern/src/lib.rs
git commit -m "Persist the tolerance, and hand-write the Default it breaks"
```

---

### ✅ Decision D6 — export's public signature — **ANSWERED 2026-08-17: option A**

**The operator chose A, project-aware.** `export_tiled_pdf(project: &Project, layout:
&PageLayout)` shipped in Task 7 (`c6ac313`). B and C are recorded below as rejected, and
**ADR-007 must carry this decision and both rejections when Task 12 writes it** — it is the
only decision in this wave that changes a public signature in a crate outside
`patal-pattern`, and a plan file is not where a decision like that should end its life.

What the answer cost in practice, for the record: 23 call sites across `export`'s own tests
and three integration test files, one `project_of` helper per test file, and the module
doc-test. No production caller outside the harness existed to migrate. Subset export is
gone until someone asks for it.

The original decision text follows unchanged.

---

Forced by **R1**, and not in the blueprint. `export_tiled_pdf(pieces: &[&PatternPiece],
layout: &PageLayout)` calls `piece.cut_boundary()` with no tolerance. After §3.6 that call
needs one, and export has nowhere to get it.

| Option | Shape | Cost |
|---|---|---|
| **A — project-aware (recommended)** | `export_tiled_pdf(project: &Project, layout: &PageLayout)` | Export reads the tolerance from the document and exports the project's pieces. Export never decides geometry; it asks `Project::cut_boundary`. Test helpers build a `Project` instead of loose pieces — mechanical, ~20 lines across the export test module. Loses the ability to export a subset until someone needs it. |
| B — tolerance parameter | `export_tiled_pdf(pieces, layout, tolerance_mm)` | Smallest diff. But it puts flattening policy in export's caller, and a caller passing a tolerance that disagrees with the document's is exactly the two-sources-of-truth failure `CutLine` exists to prevent. |
| C — both | A, plus `export_tiled_pdf_pieces(project, &[&PatternPiece], layout)` | Subsetting today, for a caller that does not exist. |

**Recommendation: A.** It is the only one where export cannot express a cut line the
document disagrees with, which is the same argument that made `CutLine` a newtype. C is
option A plus unmeasured API surface; add it when a real caller asks. Record the answer in
ADR-007's rejected section either way.

---

### Task 7: `PatternPiece` stores a `SeamPath` (§3.6) — the core change

The gap itself. `apps/desktop/src-tauri/src/lib.rs:142-148` flattens a `SeamPath` and hands
the polygon to `PatternPiece::new`; the curves are gone at that line and nothing downstream
can recover them.

**This task cannot be split.** Renaming the field breaks `patal-export` and the harness in
the same instant, so the repair lands in the same commit or the workspace does not compile.

> **Findings from executing this task (2026-08-17).** Three things this plan got wrong, and
> one prediction that was wrong in the good direction. Recorded here rather than silently
> fixed, because a plan that is only ever right is a plan nobody checked.
>
> 1. **The golden PDF did not change.** Step 5 says "**The golden PDF will change**" and
>    gives the bless command. It did not, and it should not have been expected to: Task 3's
>    property already guarantees `lift(b).flatten(t)` is bit-identical to `b`, and the piece
>    list did not reorder. The byte comparison passing untouched is a *stronger* result than
>    the plan anticipated — it extends the lift's losslessness from the geometry tests all
>    the way through the PDF writer. **Do not bless the golden for this task.**
> 2. **Step 1's offset-tightening test could not fail for the right reason.** It compared
>    `cut_boundary(0.5)` against a plain `flatten(0.5).offset(20.0)` by point count. At
>    0.5mm this curve's amplification is ~1.41, which lands inside the same
>    adaptive-subdivision jump: both routes return 17 points and the `assert_ne!` fails on
>    a correct implementation. Measured across allowance × tolerance before changing it; at
>    **0.1mm** they genuinely part, 45 against 33. The shipped test also asserts equality
>    against `flatten_for_offset` itself, so a hand-rolled fudge factor cannot pass it.
> 3. **Step 1's `a_total_perimeter_reports_failure_…` never observed a failure.** It
>    asserted a square's perimeter is 400 and stopped. The shipped version keeps that and
>    adds the case the name promises: a path running out and straight back is closed,
>    finite and constructible, and flattens to two distinct points — `TooFewPoints`, which
>    is exactly what the old infallible `-> f64` had nowhere to put.
> 4. **Step 6 says `cut_preview` routes through `Project::cut_boundary`.** It was left
>    alone. `cut_preview` calls `flatten_for_offset` on the path directly and never builds
>    a `PatternPiece`, so it did not break, and Step 6's own stated scope is "the minimum to
>    compile". Rerouting it changes what the benchmark measures and belongs with the
>    harness proof in Task 9.

**Files:**
- Modify: `engine/crates/pattern/src/lib.rs` (the piece, `cut_boundary`, `total_perimeter_mm`)
- Modify: `engine/crates/export/src/lib.rs:111-167` (per D6), `:488-540` (test helpers)
- Modify: `apps/desktop/src-tauri/src/lib.rs` (compile fixes only; the harness's own proof is Task 10)

**Interfaces:**
- Consumes: `SeamPath::from_boundary` (Task 3), `PieceId` (Task 5),
  `Project::flatten_tolerance_mm` (Task 6), `GrainLine` (Task 4).
- Produces: `PatternPiece.outline: SeamPath` (public field);
  `PatternPiece::new(name, outline: SeamPath) -> PatternPiece`;
  `PatternPiece::from_boundary(name, PatternBoundary) -> PatternPiece`;
  `PatternPiece::cut_boundary(&self, tolerance_mm: f64) -> Result<CutLine, PatternError>`;
  `PatternPiece::grain(&self) -> Option<GrainLine>` and `set_grain`;
  `Project::cut_boundary(&self, piece: &PatternPiece) -> Result<CutLine, PatternError>`;
  `Project::total_perimeter_mm(&self) -> Result<f64, PatternError>` (**R2** — now fallible);
  `export_tiled_pdf(project: &Project, layout: &PageLayout) -> Result<Vec<u8>, ExportError>`.

- [ ] **Step 1: Write the failing tests**

In `engine/crates/pattern/src/lib.rs`:

```rust
    fn square_path(side: f64) -> SeamPath {
        SeamPath::from_boundary(&square_boundary(side))
    }

    #[test]
    fn a_piece_keeps_the_curves_it_was_drawn_with() {
        // S1 and S2. The whole wave in one assertion: what goes in comes back
        // out as edges, not as a polygon someone has to re-guess.
        let start = Point2::new(0.0, 0.0);
        let outline = SeamPath::closed(
            start,
            vec![
                EdgeSegment::Cubic {
                    c1: Point2::new(15.0, -30.0),
                    c2: Point2::new(50.0, -22.0),
                    to: Point2::new(75.0, 10.0),
                },
                EdgeSegment::Line { to: Point2::new(75.0, 100.0) },
                EdgeSegment::Line { to: start },
            ],
        )
        .expect("closes");

        let piece = PatternPiece::new("Bodice Front", outline.clone());
        let json = serde_json::to_string(&piece).expect("serializes");
        let restored: PatternPiece = serde_json::from_str(&json).expect("round trips");

        assert_eq!(restored.outline, outline);
        assert_eq!(restored.outline.edges().len(), 3);
        assert_eq!(
            restored
                .outline
                .edges()
                .iter()
                .filter(|e| matches!(e.geometry(), EdgeSegment::Cubic { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn no_polygon_appears_anywhere_in_a_serialized_piece() {
        // S3. The derived boundary is derived, never persisted — otherwise a
        // file can assert an outline that disagrees with its own curves.
        let piece = PatternPiece::new("Front", square_path(200.0));
        let json = serde_json::to_string(&piece).expect("serializes");
        assert!(!json.contains("boundary"), "a polygon reached the wire: {json}");
        assert!(json.contains("outline"));
    }

    #[test]
    fn a_project_supplies_its_own_tolerance_to_the_cut_line() {
        // S5. Two routes to the same answer, so the ergonomic one cannot
        // drift from the testable one.
        let mut project = Project::new("Blouse");
        project.set_flatten_tolerance_mm(0.02).expect("valid");
        let piece = PatternPiece::new("Front", square_path(200.0));

        let via_project = project.cut_boundary(&piece).expect("cuts");
        let via_piece = piece.cut_boundary(0.02).expect("cuts");
        assert_eq!(via_project.points(), via_piece.points());
    }

    #[test]
    fn the_cut_line_is_flattened_against_the_offset_it_is_about_to_receive() {
        // The correctness upgrade this wave gets in passing. Plain flatten
        // discretises with no knowledge of the impending offset, which is
        // exactly the error flatten_for_offset exists to prevent. On a curved
        // piece with a large allowance the two disagree; if they ever stop
        // disagreeing, cut_boundary has quietly regressed to plain flatten.
        let start = Point2::new(0.0, 0.0);
        let outline = SeamPath::closed(
            start,
            vec![
                EdgeSegment::Cubic {
                    c1: Point2::new(10.0, 60.0),
                    c2: Point2::new(90.0, 60.0),
                    to: Point2::new(100.0, 0.0),
                },
                EdgeSegment::Line { to: start },
            ],
        )
        .expect("closes");

        let mut piece = PatternPiece::new("Curved", outline.clone());
        piece.set_seam_allowance_mm(20.0).expect("valid");

        let tolerance = 0.5;
        let tight = piece.cut_boundary(tolerance).expect("cuts");
        let naive = outline
            .flatten(tolerance)
            .expect("flattens")
            .offset(20.0)
            .expect("offsets");

        assert_ne!(
            tight.points().len(),
            naive.points().len(),
            "cut_boundary must tighten for the offset, not flatten blind"
        );
    }

    #[test]
    fn a_total_perimeter_reports_failure_rather_than_a_plausible_number() {
        // R2: the signature is fallible now because flattening is.
        let mut project = Project::new("Blouse");
        project.add_piece(PatternPiece::new("Front", square_path(100.0)));
        let total = project.total_perimeter_mm().expect("flattens");
        assert!((total - 400.0).abs() < 1e-9, "got {total}");
    }
```

- [ ] **Step 2: Run to verify it fails**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-pattern'
```

Expected: FAIL to compile — `PatternPiece::new` still takes a `PatternBoundary`.

- [ ] **Step 3: Change the piece**

In `engine/crates/pattern/src/lib.rs`, replace `boundary: PatternBoundary` with
`outline: SeamPath` on both `PatternPiece` and `PatternPieceData`, add
`grain: Option<GrainLine>` with `#[serde(default)]` on the data shape, and rewrite:

```rust
    /// Builds a piece from the path the designer drew.
    ///
    /// `outline` is public for the same reason `boundary` was: a `SeamPath`
    /// cannot be constructed invalid, so assignment cannot smuggle in a bad
    /// value.
    pub fn new(name: impl Into<String>, outline: SeamPath) -> Self {
        Self {
            name: name.into(),
            outline,
            id: PieceId::new(),
            seam_allowance_mm: Self::DEFAULT_SEAM_ALLOWANCE_MM,
            material: None,
            grain: None,
        }
    }

    /// Builds a piece from a polygon, lifting it into an all-corner path.
    ///
    /// Every caller that used to hand over a `PatternBoundary` migrates in one
    /// line. The migration in `migrate.rs` uses this too.
    pub fn from_boundary(name: impl Into<String>, boundary: PatternBoundary) -> Self {
        Self::new(name, SeamPath::from_boundary(&boundary))
    }

    /// The outline including seam allowance — what actually gets cut.
    ///
    /// The one place in the codebase a [`CutLine`] comes into existence.
    ///
    /// Flattens through `flatten_for_offset`, not plain `flatten`: the
    /// discretisation has to hold *after* the offset, and a boundary
    /// flattened with no knowledge of the impending offset is precisely the
    /// error that function exists to prevent.
    ///
    /// A curve that succeeds at 0.01mm and fails at 0.001mm with
    /// `OffsetSelfIntersects` is **correct behaviour, not a regression**: a
    /// chord next to a sharp corner has become shorter than the allowance, and
    /// the loud failure is the right answer. Do not weaken the check.
    pub fn cut_boundary(&self, tolerance_mm: f64) -> Result<CutLine, PatternError> {
        let flattened = self
            .outline
            .flatten_for_offset(tolerance_mm, self.seam_allowance_mm)?;
        Ok(CutLine {
            piece: self.name.clone(),
            boundary: flattened.offset(self.seam_allowance_mm)?,
        })
    }

    pub fn grain(&self) -> Option<GrainLine> {
        self.grain
    }

    pub fn set_grain(&mut self, grain: Option<GrainLine>) {
        self.grain = grain;
    }
```

**No `#[serde(skip)]` boundary cache in this wave.** A cache is unmeasured optimisation;
Task 12 measures, and it lands only if the measurement asks for it.

On `Project`:

```rust
    /// [`PatternPiece::cut_boundary`] at this project's tolerance.
    ///
    /// Two functions rather than a piece-to-project back-reference: the piece
    /// stays testable in isolation, and the project stays the ergonomic path.
    pub fn cut_boundary(&self, piece: &PatternPiece) -> Result<CutLine, PatternError> {
        piece.cut_boundary(self.flatten_tolerance_mm)
    }

    /// Plain `flatten`, not `flatten_for_offset`: nothing is being offset
    /// here, so tightening would be wrong.
    pub fn total_perimeter_mm(&self) -> Result<f64, PatternError> {
        let mut total = 0.0;
        for piece in &self.pieces {
            total += piece.outline.flatten(self.flatten_tolerance_mm)?.perimeter();
        }
        Ok(total)
    }
```

Update `TryFrom<PatternPieceData>` and `From<PatternPiece>` to carry `outline`, `id` and
`grain`. Then sweep the remaining internal uses of `self.boundary`
(`pattern/src/lib.rs:124,134,138` belong to `CutLine` and do **not** change — that is the
cut line's own boundary, not the piece's).

- [ ] **Step 4: Find every break**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat check --workspace --all-targets --locked'
```

Work the error list. Expect breaks in `engine/crates/export/src/lib.rs` and
`apps/desktop/src-tauri/src/lib.rs`, plus `total_perimeter_mm` call sites. Do not silence
anything with `.unwrap()` in library code.

- [ ] **Step 5: Make export project-aware (per D6-A)**

`engine/crates/export/src/lib.rs`:

```rust
pub fn export_tiled_pdf(project: &Project, layout: &PageLayout) -> Result<Vec<u8>, ExportError> {
    if project.pieces.is_empty() {
        return Err(ExportError::NothingToExport);
    }

    let mut plans: Vec<PiecePlan> = Vec::with_capacity(project.pieces.len());
    for piece in &project.pieces {
        let cut = project
            .cut_boundary(piece)
            .map_err(|source| ExportError::CutLineFailed {
                piece: piece.name.clone(),
                source,
            })?;

        // The sewing line is the piece's own outline at the project's
        // tolerance — not a re-derivation of anything, and not a second
        // opinion about the cut.
        let sewing = piece
            .outline
            .flatten(project.flatten_tolerance_mm())
            .map_err(|source| ExportError::CutLineFailed {
                piece: piece.name.clone(),
                source: source.into(),
            })?;

        let bounds =
            BoundsMm::of_boundary(cut.boundary()).union(BoundsMm::of_boundary(&sewing));
        let grid = TileGrid::cover(bounds, *layout);
        // ... TooManyTiles check unchanged ...
        plans.push(PiecePlan {
            name: piece.name.clone(),
            seam_allowance_mm: piece.seam_allowance_mm(),
            cut,
            sewing,
            grid,
        });
    }
    // ... phase two unchanged ...
}
```

Update the module doc-comment example at `lib.rs:44-60` to build a `Project`, since it is a
compiled doc-test. Update `rect_piece` and `notched_piece` (`:488`, `:533`) to return pieces
built with `PatternPiece::from_boundary`, and add a helper that wraps pieces in a `Project`
at the default tolerance for the export tests to call.

~~**The golden PDF will change**~~ — **it did not. Do not bless it.** The reasoning below
was that the sewing line is now flattened from a lifted path rather than taken directly and
the calibration page's piece list may reorder. Task 3's property makes the first half
bit-identical and the second never happened, so the golden passed untouched on 2026-08-17.
Kept here because the command is still the right one if a *later* task legitimately moves
the bytes — and because a golden that changes during this task means something is wrong,
not that it needs re-blessing:

```bash
PATAL_BLESS_GOLDEN=1 cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-export'
```

- [ ] **Step 6: Fix the harness to compile**

In `apps/desktop/src-tauri/src/lib.rs`, the minimum to compile: `PatternPiece::new` now
takes the `SeamPath` directly, so the `flatten` call at line 143 is deleted, and
`cut_preview` routes through `Project::cut_boundary`. The harness's *proof* — reporting
segment and cubic counts — is Task 10; this step only restores the build.

- [ ] **Step 7: Full gate**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat fmt --check'
cmd //c 'C:\Users\User\patal\scripts\cargo.bat clippy --workspace --all-targets --locked -- -D warnings'
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --workspace --locked'
PATAL_CARGO_DIR=C:\Users\User\patal\apps\desktop\src-tauri cmd //c 'C:\Users\User\patal\scripts\cargo.bat clippy --all-targets --locked -- -D warnings'
```

Expected: PASS, ~~166~~ **168 tests** (the baseline moved +2 when PR #5 merged; see the
execution status header). Every one of the 18 original `patal-pattern` tests must still
be present — updated only where the *type* changed, never where the behaviour should have
held. If a test's assertion had to be relaxed to pass, stop: that is a regression wearing a
test change as a disguise.

- [ ] **Step 8: Commit**

```bash
git add engine/crates/pattern/src/lib.rs engine/crates/export/src/lib.rs \
        apps/desktop/src-tauri/src/lib.rs
git commit -m "Store the curve the designer drew, not the polygon it flattens to"
```

---

### ⛔ GATE — the v2 shape freeze (§3.7 approval node)

**Stop here. Do not start Task 8 without an explicit operator sign-off.**

This is a one-way door. Once the migration is written against a shape, changing the shape
means changing the migration. It is cheap to pause here precisely because Tasks 1–7 are all
additive or internal — nothing written so far has reached a file that anyone holds.

Print the exact v2 document shape and hand it over:

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-pattern -- --nocapture print_v2_shape'
```

Add a temporary `#[test] fn print_v2_shape()` that serialises a two-piece project with one
cubic edge, one `Smooth` join, a grain line and a non-default tolerance, and prints it
pretty. Delete the test once the shape is signed off — it is a review instrument, not a
regression.

**What the operator is signing off, stated plainly:**

1. A piece stores `outline` (a `SeamPath`) and **never** a polygon.
2. An edge is `{"geometry": {...}, "join": "..."}` — nested, not flat.
3. `join` may be omitted and means `corner`; `geometry` may not be omitted.
4. A piece carries `id` (bare UUID string), `grain` (nullable), `seam_allowance_mm`, `material`.
5. A project carries `flatten_tolerance_mm`, defaulting to 0.01.
6. **What is deliberately absent:** per-edge seam allowance (P-03), fold edges (P-05),
   notch anchors (P-13). These stay unresolved and land at K6 on evidence. The `Edge`
   container is what makes each of them a field on an existing struct rather than a schema
   v3 — that is the entire argument for revision 6, and this is the moment it either holds
   or does not.

**Also still open, and not resolved by this freeze:** whether a dart is an object
(Decision 2 of the census). It is blocked on K3 — hand-drafting a block in Seamly2D and
Freesewing — and freezing v2 does not decide it. If a dart turns out to be an object rather
than a derived operation, it is an *additive* piece-level field, not a change to any shape
above. Confirm that reading before signing.

---

### Task 8: Schema v2 and the migration (§3.7)

Build the migration mechanism once, on a case where being wrong is free. **D3 is the hedge
that makes being wrong about D1 survivable** — that is the strongest argument for having
taken it, and it is why this task exists at all rather than simply bumping the version.

**Files:**
- Create: `engine/crates/pattern/src/migrate.rs`
- Create: `fixtures/v1-bodice.patal`, `fixtures/v2-bodice.patal`
- Modify: `engine/crates/pattern/src/lib.rs` (`SCHEMA_VERSION`, `Document`'s loader)

**Interfaces:**
- Consumes: everything from Tasks 1–7.
- Produces: `SCHEMA_VERSION = 2`;
  `pub fn migrate_v1(...) -> Result<Document, PatternError>` (pure);
  `PatternError::MigrationFailed { from: u32, reason: String }`.
  `Document`'s public API is unchanged — `Document::new`, `schema_version()`.

- [ ] **Step 1: Write the v1 fixture by hand**

`fixtures/v1-bodice.patal`. Hand-written, not generated, and **frozen forever** — it is a
historical record of what v1 files looked like, and regenerating it from current code would
destroy the only evidence the migration is tested against something real.

```json
{
  "schema_version": 1,
  "project": {
    "name": "Bodice",
    "pieces": [
      {
        "name": "Bodice Front",
        "boundary": [
          {"x": 0.0, "y": 0.0},
          {"x": 200.0, "y": 0.0},
          {"x": 200.0, "y": 300.0},
          {"x": 0.0, "y": 300.0}
        ],
        "seam_allowance_mm": 12.0,
        "material": null
      }
    ],
    "measurements": [{"name": "bust", "value_mm": 920.0}],
    "materials": {"materials": []}
  }
}
```

Verify the `materials` key matches `MaterialLibrary`'s actual wire shape before committing
— read it from `engine/crates/materials/src/lib.rs` rather than guessing.

- [ ] **Step 2: Write the failing tests**

Create `engine/crates/pattern/tests/migration.rs`:

```rust
use std::path::PathBuf;

use patal_pattern::{Document, PatternError, SCHEMA_VERSION};

fn fixture(name: &str) -> String {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "..", "..", "fixtures", name]
        .iter()
        .collect();
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {path:?}: {e}"))
}

#[test]
fn a_v1_document_migrates_rather_than_being_refused() {
    // S6. Refusing it would be defensible; migrating it is what makes being
    // wrong about the v2 shape survivable later.
    let document: Document = serde_json::from_str(&fixture("v1-bodice.patal"))
        .expect("a v1 file loads");
    assert_eq!(document.schema_version(), SCHEMA_VERSION);
    assert_eq!(document.project.pieces.len(), 1);
}

#[test]
fn a_migrated_outline_flattens_bit_identical_to_the_v1_polygon() {
    // The losslessness claim, and it is bit-exact for the same reason the
    // lift's property is: no arithmetic happens on the way through.
    let document: Document =
        serde_json::from_str(&fixture("v1-bodice.patal")).expect("loads");
    let piece = &document.project.pieces[0];
    let flattened = piece
        .outline
        .flatten(document.project.flatten_tolerance_mm())
        .expect("flattens");

    let expected = [
        (0.0, 0.0), (200.0, 0.0), (200.0, 300.0), (0.0, 300.0),
    ];
    let actual: Vec<(f64, f64)> = flattened.points().iter().map(|p| (p.x, p.y)).collect();
    assert_eq!(actual.len(), expected.len());
    for (got, want) in actual.iter().zip(expected.iter()) {
        assert_eq!(got, want, "bit-identical, not within-epsilon");
    }
}

#[test]
fn migration_preserves_what_v1_actually_said() {
    let document: Document =
        serde_json::from_str(&fixture("v1-bodice.patal")).expect("loads");
    let piece = &document.project.pieces[0];
    assert_eq!(piece.name, "Bodice Front");
    assert_eq!(piece.seam_allowance_mm(), 12.0);
    assert_eq!(document.project.measurement("bust"), Some(920.0));
    // Minted, not read: no v1 file references a piece by id, so minting is safe.
    assert!(piece.grain().is_none());
}

#[test]
fn a_v1_document_carrying_a_v2_field_is_refused_rather_than_half_read() {
    let mut json: serde_json::Value =
        serde_json::from_str(&fixture("v1-bodice.patal")).expect("parses");
    json["project"]["pieces"][0]["outline"] =
        serde_json::json!({"start": {"x": 0.0, "y": 0.0}, "edges": []});
    let err = serde_json::from_value::<Document>(json).expect_err("refused");
    assert!(err.to_string().contains("outline"), "{err}");
}

#[test]
fn a_v2_document_missing_its_outline_names_the_field_rather_than_failing_to_parse() {
    let mut json: serde_json::Value =
        serde_json::from_str(&fixture("v2-bodice.patal")).expect("parses");
    json["project"]["pieces"][0]
        .as_object_mut()
        .unwrap()
        .remove("outline");
    let err = serde_json::from_value::<Document>(json).expect_err("refused");
    assert!(err.to_string().contains("outline"), "{err}");
}

#[test]
fn a_future_schema_is_refused_with_a_sentence_not_a_parse_error() {
    let mut json: serde_json::Value =
        serde_json::from_str(&fixture("v2-bodice.patal")).expect("parses");
    json["schema_version"] = serde_json::json!(3);
    let err = serde_json::from_value::<Document>(json).expect_err("refused");
    let message = err.to_string();
    assert!(message.contains("newer version"), "{message}");
    assert!(
        !message.contains("did not match any variant"),
        "this is the untagged failure mode the version field exists to avoid: {message}"
    );
}

#[test]
fn a_v2_document_round_trips_through_the_hand_written_loader_unchanged() {
    // §5.4's fallback, made permanent. The hand-written impl must not drift
    // from what the derive would have accepted for v2.
    let original = fixture("v2-bodice.patal");
    let document: Document = serde_json::from_str(&original).expect("loads");
    let reserialized = serde_json::to_string_pretty(&document).expect("serializes");
    let reloaded: Document = serde_json::from_str(&reserialized).expect("reloads");
    assert_eq!(
        serde_json::to_string_pretty(&reloaded).unwrap(),
        reserialized
    );
}
```

- [ ] **Step 3: Run to verify it fails**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-pattern --test migration'
```

Expected: FAIL — the v1 fixture is refused by `UnsupportedSchemaVersion`, and
`fixtures/v2-bodice.patal` does not exist yet.

- [ ] **Step 4: Write the version-tolerant loader (per R6)**

Create `engine/crates/pattern/src/migrate.rs`:

```rust
//! Reading documents older than this build writes.
//!
//! # Why this is not a hand-written `Deserialize` that reads the version first
//!
//! The obvious design reads `schema_version` from the map, then dispatches to
//! a per-version project shape. It cannot be written safely here. JSON does
//! not guarantee key order, so `schema_version` may arrive *after* `project`;
//! dispatching correctly would mean buffering the map, and the two tools for
//! that are both closed to this crate — `serde_json::Value` is a
//! dev-dependency only, and buffering through it would drag a format-specific
//! type into a format-agnostic crate, while `serde`'s own `Content` is private
//! API.
//!
//! So the shapes below are version-*tolerant* rather than version-*specific*:
//! every field either version can carry, with the version-specific ones
//! optional, and a strict dispatch afterwards that refuses a document carrying
//! the wrong version's fields. Same guarantees, no dependence on key order.
//!
//! # These shapes are frozen
//!
//! `AnyPieceData` is a historical record of every field any released version
//! has written. Editing it silently changes what old files mean. Add to it
//! when a version ships; never repurpose or remove.

use patal_geometry::{PatternBoundary, Point2};
use patal_materials::{MaterialId, MaterialLibrary};
use serde::Deserialize;

use crate::{
    Document, GrainLine, Measurement, PatternError, PatternPiece, PieceId, Project, SeamPath,
    DEFAULT_FLATTEN_TOLERANCE_MM, SCHEMA_VERSION,
};

#[derive(Deserialize)]
pub(crate) struct AnyDocumentData {
    schema_version: u32,
    project: AnyProjectData,
}

#[derive(Deserialize)]
struct AnyProjectData {
    name: String,
    pieces: Vec<AnyPieceData>,
    measurements: Vec<Measurement>,
    #[serde(default)]
    materials: MaterialLibrary,
    /// v2 onward. Absent in v1 and absent in a hand-edited v2.
    #[serde(default)]
    flatten_tolerance_mm: Option<f64>,
}

#[derive(Deserialize)]
struct AnyPieceData {
    name: String,
    seam_allowance_mm: f64,
    #[serde(default)]
    material: Option<MaterialId>,
    /// v1 only.
    #[serde(default)]
    boundary: Option<PatternBoundary>,
    /// v2 onward.
    #[serde(default)]
    outline: Option<SeamPath>,
    #[serde(default)]
    id: Option<PieceId>,
    #[serde(default)]
    grain: Option<GrainLine>,
}

impl TryFrom<AnyDocumentData> for Document {
    type Error = PatternError;

    fn try_from(data: AnyDocumentData) -> Result<Self, Self::Error> {
        match data.schema_version {
            1 => migrate_v1(data.project),
            2 => load_v2(data.project),
            found => Err(PatternError::UnsupportedSchemaVersion {
                found,
                supported: SCHEMA_VERSION,
            }),
        }
    }
}

/// v1 → v2, as a **pure function**: it returns a new document or an error and
/// never mutates one in place. A half-migrated document that a caller then
/// saves is how a read bug becomes a write bug.
fn migrate_v1(data: AnyProjectData) -> Result<Document, PatternError> {
    let mut project = Project::new(data.name);
    project.measurements = data.measurements;
    project.materials = data.materials;
    // v1 had no tolerance. The default is the only honest answer — there is
    // no value in the file to preserve.
    project
        .set_flatten_tolerance_mm(DEFAULT_FLATTEN_TOLERANCE_MM)
        .expect("the crate's own default is valid");

    for piece_data in data.pieces {
        if piece_data.outline.is_some() {
            return Err(PatternError::MigrationFailed {
                from: 1,
                reason: format!(
                    "piece \"{}\" carries an `outline`, which no version 1 document \
                     can contain. This file claims version 1 and is not one.",
                    piece_data.name
                ),
            });
        }
        let Some(boundary) = piece_data.boundary else {
            return Err(PatternError::MigrationFailed {
                from: 1,
                reason: format!(
                    "piece \"{}\" has no `boundary`, which every version 1 piece has",
                    piece_data.name
                ),
            });
        };

        // The lift produces edges already carrying `Join::Corner`, so there is
        // no second array to build to the right length and no way to build it
        // wrong. Under the parallel-array design this was a mapping step.
        let mut piece = PatternPiece::from_boundary(piece_data.name, boundary);
        piece.set_seam_allowance_mm(piece_data.seam_allowance_mm)?;
        piece.material = piece_data.material;
        // `id` is freshly minted by `from_boundary`. Safe: no v1 file
        // references a piece by id, so nothing can be left dangling.
        project.add_piece(piece);
    }

    project.check_material_references()?;
    Ok(Document::new(project))
}

fn load_v2(data: AnyProjectData) -> Result<Document, PatternError> {
    let mut project = Project::new(data.name);
    project.measurements = data.measurements;
    project.materials = data.materials;
    project.set_flatten_tolerance_mm(
        data.flatten_tolerance_mm.unwrap_or(DEFAULT_FLATTEN_TOLERANCE_MM),
    )?;

    for piece_data in data.pieces {
        if piece_data.boundary.is_some() {
            return Err(PatternError::MigrationFailed {
                from: 2,
                reason: format!(
                    "piece \"{}\" carries a `boundary`. Version 2 stores an `outline` \
                     and derives the polygon; a file asserting both can disagree with \
                     itself.",
                    piece_data.name
                ),
            });
        }
        let Some(outline) = piece_data.outline else {
            return Err(PatternError::MigrationFailed {
                from: 2,
                reason: format!("piece \"{}\" has no `outline`", piece_data.name),
            });
        };

        let mut piece = PatternPiece::new(piece_data.name, outline);
        piece.set_seam_allowance_mm(piece_data.seam_allowance_mm)?;
        piece.material = piece_data.material;
        piece.set_grain(piece_data.grain);
        if let Some(id) = piece_data.id {
            piece.set_id_on_load(id);
        }
        project.add_piece(piece);
    }

    project.check_material_references()?;
    Ok(Document::new(project))
}
```

`set_id_on_load` is `pub(crate)` on `PatternPiece` — the *only* way an id is ever assigned
rather than minted, and crate-private so no caller outside the loader can claim one.

In `lib.rs`: set `SCHEMA_VERSION = 2`, update its doc comment to say what v2 changed, point
`Document`'s serde at the new shape with
`#[serde(try_from = "crate::migrate::AnyDocumentData", into = "DocumentData")]`, add
`mod migrate;`, and add the error variant:

```rust
    /// A document that names a version it does not match.
    MigrationFailed { from: u32, reason: String },
```

with a `Display` arm that prints `reason` verbatim and a `source()` arm returning `None`.

- [ ] **Step 5: Generate the v2 fixture, then freeze it**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --package patal-pattern -- --nocapture write_v2_fixture'
```

Write a temporary test that builds the same bodice as the v1 fixture *plus* one cubic edge,
one `Smooth` join and a grain line, serialises it pretty, and writes
`fixtures/v2-bodice.patal`. Then **delete the test** and commit the file. Both Rust and
Swift read this one file, so the two languages are pinned to a fixture rather than to each
other.

- [ ] **Step 6: Run and commit**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --workspace --locked'
```

Expected: PASS, **173 tests**.

```bash
git add engine/crates/pattern/src/migrate.rs engine/crates/pattern/src/lib.rs \
        engine/crates/pattern/tests/migration.rs fixtures/
git commit -m "Read version 1 files rather than refusing them"
```

---

### Task 9: The harness proves the curves came back (§3.8)

The gap was visible in the harness, so the fix is proven there. Task 7 only restored the
build; this is the demonstration.

**Files:** Modify `apps/desktop/src-tauri/src/lib.rs`.

**Interfaces:**
- Consumes: Tasks 7 and 8.
- Produces: `SaveReport` gains `edges: usize`, `cubics: usize`; a
  `load_v1_fixture(path: String) -> Result<MigrationReport, String>` Tauri command.

- [ ] **Step 1: Write the failing test**

The harness has real tests — it is the only place in the repo that exercises the document
format through an actual file. Add to its test module:

```rust
    #[test]
    fn a_saved_document_reloads_as_curves_not_as_a_polygon() {
        let dir = tempdir().expect("temp dir");
        let report = save_demo_document(dir.path().display().to_string(), 0.01)
            .expect("saves");

        assert!(report.round_tripped);
        assert_eq!(report.schema_version, 2);
        // A vertex count proves nothing — a flattened polygon has plenty of
        // vertices. Edge and cubic counts are what say the curves survived.
        assert_eq!(report.edges, 6);
        assert_eq!(report.cubics, 4);
    }
```

Use whatever temp-directory approach the harness's existing tests already use rather than
adding a dependency.

- [ ] **Step 2: Implement**

Delete the flatten at `lib.rs:143` so `bodice_front()` reaches `PatternPiece::new` intact —
**that deleted line is the entire wave in one edit.** Add `edges` and `cubics` to
`SaveReport`, computed off the *reloaded* piece:

```rust
    let edges = reloaded_piece.outline.edges().len();
    let cubics = reloaded_piece
        .outline
        .edges()
        .iter()
        .filter(|e| matches!(e.geometry(), EdgeSegment::Cubic { .. }))
        .count();
```

Route `cut_preview` through `Project::cut_boundary` so the harness exercises the real path
rather than a parallel one, and add the `load_v1_fixture` command that reads
`fixtures/v1-bodice.patal` and reports what it migrated to. Register both commands in the
Tauri builder and surface the new numbers in the frontend, so "Reloaded 6 edges, 4 cubic" is
visible without reading a log.

- [ ] **Step 3: Run and commit**

```bash
PATAL_CARGO_DIR=C:\Users\User\patal\apps\desktop\src-tauri cmd //c 'C:\Users\User\patal\scripts\cargo.bat test --locked'
PATAL_CARGO_DIR=C:\Users\User\patal\apps\desktop\src-tauri cmd //c 'C:\Users\User\patal\scripts\cargo.bat clippy --all-targets --locked -- -D warnings'
```

```bash
git add apps/desktop/
git commit -m "Show the curves surviving the file, not just the bytes"
```

---

### ⛔ GATE — Swift: mirror or delete (§3.9)

**Operator's call. Blueprint §6 frames it; do not decide it yourself.**

- **Mirror.** Add `EdgeSegment`, `Edge`, `Join`, `SeamPath`, `GrainLine` and the engine's
  `PieceId` as Codable value types — roughly 60 lines plus one small struct for `Edge`.
  Unlike the deleted offset kernel there is no divergent-behaviour risk: a Codable struct
  either decodes the snake_case shape or it does not.
- **Delete.** Last wave's argument applies unchanged — Swift's `PatternPiece` has no
  reachable consumer, there is no Xcode project, and `Project.swift:82-88` calls itself
  Swift-to-Swift only. Deleting costs nothing today and removes the tax permanently.

**Blueprint recommendation: mirror**, because the deleted code was a second *implementation
of geometry* and this is a data model. Revision 6 strengthens it slightly: `Edge` removes a
two-arrays-same-length check Swift had no good way to enforce.

**Leaving it stale is the one answer that is definitely wrong** — CI gates on the `native`
job, and `Project.swift` decodes `boundary`, which v2 no longer writes.

---

### Task 10: Swift mirrors the v2 shape (§3.9, if "mirror")

**Files:** Modify `apps/native/Sources/PatalKit/Models/Geometry.swift`,
`Models/Project.swift`, `Tests/PatalKitTests/PatalKitTests.swift`.

- [ ] **Step 1: Write the failing test**

```swift
func testDecodesTheSameV2FixtureTheEngineWrote() throws {
    // Both languages pin to one file rather than to each other. If the engine
    // changes the shape, this fails on the next CI run rather than at the
    // first attempt to open a real document.
    let root = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent()  // PatalKitTests
        .deletingLastPathComponent()  // Tests
        .deletingLastPathComponent()  // native
        .deletingLastPathComponent()  // apps
    let url = root.appendingPathComponent("fixtures/v2-bodice.patal")

    let document = try JSONDecoder().decode(Document.self, from: Data(contentsOf: url))
    let piece = try XCTUnwrap(document.project.pieces.first)

    XCTAssertEqual(document.schemaVersion, 2)
    XCTAssertFalse(piece.outline.edges.isEmpty)
    XCTAssertTrue(piece.outline.edges.contains { if case .cubic = $0.geometry { return true }; return false })

    // Re-encode and decode again: the shape survives a Swift round trip.
    let reencoded = try JSONEncoder().encode(document)
    _ = try JSONDecoder().decode(Document.self, from: reencoded)
}
```

- [ ] **Step 2: Implement**

Add to `Geometry.swift`: `EdgeSegment` as an enum with a `kind` discriminator matching
Rust's `#[serde(tag = "kind", rename_all = "snake_case")]`; `Join` as a `String`-raw-value
enum defaulting to `.corner` when the key is absent; `Edge` as a struct with `geometry` and
`join`; `SeamPath` with `start` and `edges`.

In `Project.swift`: replace `boundary` with `outline: SeamPath`, add `grain: GrainLine?`,
change `id` from a Swift invention to the engine's value, and **delete the "Swift-to-Swift
only" comment at `:82-88`** — Task 5 is what retires it. Add `outline` and `grain` to
`CodingKeys` in snake_case.

**Swift does not re-validate.** No tangent check for `Join.smooth`, no closure check for
`SeamPath`. Rust owns validation; a second validator is a second implementation that can
disagree, which is the exact failure the kernel deletion removed. Swift decodes what the
engine wrote and trusts it. **Value types only — no algorithms, no geometry, no flattening.
If a Swift function would need a tolerance argument, it does not belong here.**

- [ ] **Step 3: Verify**

`swift build` cannot run on this machine — there is no macOS toolchain. **CI is the only
check.** Push and watch:

```bash
gh pr checks <n> --watch --interval 15
```

The `native` job must go green before this task is done. Assume every Swift change is
unverified until it does.

- [ ] **Step 4: Commit**

```bash
git add apps/native/
git commit -m "Mirror the v2 shape in Swift, and nothing else"
```

---

### Task 11: Benchmarks measure the no-cache decision (§3.11)

§3.6 declined a `#[serde(skip)]` boundary cache on the grounds that it is unmeasured
optimisation. This is where that call either holds or does not — and it rests on last
wave's measurement of *one* piece, which is why the 50-piece case exists.

**Files:** ~~Modify `engine/crates/geometry/benches/drag_loop.rs`.~~
**DEVIATION, corrected 2026-08-17:** create `engine/crates/pattern/benches/cache_decision.rs`
instead. `PatternPiece` and `Project` are `patal-pattern`'s types and `patal-pattern`
already depends on `patal-geometry`, so benching them from the geometry bench would
require `patal-geometry` to dev-depend on `patal-pattern`. Cargo permits that cycle, but
it inverts the one dependency direction the crate split exists to enforce and pulls the
whole pattern layer into the kernel's bench build. `drag_loop` stays kernel-only.

**A second deviation, and it changed the answer.** This task as written measures only
`total_perimeter_mm`, which takes the *cheap* path — plain `flatten`, no offset, no O(n²)
self-intersection test. A boundary cache would serve the *expensive* path, so the bench
also measures every piece's `cut_boundary` at document scale. That case is the one that
exceeds a frame (8.77ms at 50 pieces); measuring only what this task specified would have
missed it entirely. See the bench header for the full table and the conclusion.

- [ ] **Step 1: Extend the bench**

Add a case measuring through `PatternPiece::cut_boundary` at the default 0.01mm tolerance,
and a `total_perimeter_mm` case at **50 pieces** — that is the call which flattens every
piece with no cache behind it, and the one place the decision could be wrong.

- [ ] **Step 2: Run and read the number**

```bash
cmd //c 'C:\Users\User\patal\scripts\cargo.bat bench --package patal-geometry'
```

The budget is a 120Hz frame: **8.33ms**. Record the actual figures in the commit message.
If the 50-piece perimeter case exceeds a few percent of a frame, the cache decision was
wrong — say so, and open a follow-up rather than silently adding the cache here.

- [ ] **Step 3: Commit**

```bash
git add engine/crates/geometry/benches/drag_loop.rs
git commit -m "Measure the cache we did not build"
```

---

### Task 12: ADR-007 and the doc close-out (§3.12)

**Files:** Create `docs/adr/ADR-007-what-a-pattern-piece-stores.md`; modify
`docs/adr/README.md`, `docs/adr/ADR-004-document-format.md`,
`docs/adr/ADR-003-curve-representation.md`, `docs/status.md`, `README.md`, `TODOS.md`.

- [ ] **Step 1: Write ADR-007**

Follow the format the existing six use: frontmatter with `id`/`title`/`status`/`date`, then
Context, Decision, Consequences. Record:

- **D1** a piece stores `SeamPath` only, polygon derived; **D2** tolerance project-level and
  persisted; **D3** a real v1→v2 migration; **D4** the three fold-ins.
- **The C9 argument for the lift** — why an explicit closing edge on a polygon that was
  already closed is not invented geometry.
- **The validated-`Smooth` veto**, in the rejected section. An unchecked continuity flag was
  drafted and refused under C1.
- **The tolerance default with its measurement**, not merely its value.
- **The loader-dispatch rejection of `untagged`**, and — **new, per R6** — why the
  read-the-version-first design was replaced by an order-independent tolerant shape.
- **D6**, export's project-aware signature, with the rejected options.
- **The edge-attribute container**, and in the rejected section the parallel-array shape it
  replaced. **This one matters more than it looks.** The reason for `Edge` is not visible in
  the code it produces: a struct holding one field reads like indirection for its own sake,
  and the next reader will be tempted to flatten it back. What justifies it is the three
  attributes not yet added — cite census rows **P-03, P-05 and P-13 by number** so the
  argument survives the wave that made it.

- [ ] **Step 2: Close the open items**

In `docs/adr/README.md`, add the 007 row and delete its "not yet written" entry — leaving
**006 alone**, still reserved and still unwritten, because it is blocked on K3 and must not
be written from a census. In ADR-004, close both open items: the flattened-boundary note
(this wave) and the piece identity divergence (Task 5). In ADR-003, add a back-reference —
the two-layer split now reaches the document.

- [ ] **Step 3: Refresh status and README**

`docs/status.md` is the single source of truth and must name: the new test count, that the
PDF has **still never been printed**, that **no pattern maker has been found**, and that
**Decision 2 — is a dart an object? — remains open and blocked on K3**. Do not let a large
green wave read as though the two tests that count have moved. They have not.

In `TODOS.md`, update the nesting entry: its stated dependency, "grain line landing", is met
by Task 4 — and record that before this wave the dependency was *not* met despite the
document implying otherwise.

- [ ] **Step 4: Commit**

```bash
git add docs/ README.md TODOS.md
git commit -m "Write ADR-007, and close what it settles"
```

---

## Acceptance criteria

Carried from blueprint §7, with the count updated and three rows added for this plan's
reconciliation findings.

- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings` clean
- [ ] `cargo test --workspace --locked` green, count reported and **> 136** (projected ~173)
- [ ] `cargo deny check` clean
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` clean
- [ ] `apps/desktop` compiles under `-D warnings`
- [ ] CI `native` job green
- [ ] All **30** pre-existing `geometry/src/lib.rs` tests pass **unmodified** (`git diff --stat` on that file shows only the two error variants, their `Display` arms, and the one-line re-export)
  - *Corrected 2026-08-16.* The blueprint says 31 and this plan copied it; the tree says 30, on `main` and on the wave branch alike, and `tests/properties.rs`'s own header has said 30 all along. Verified by counting `#[test]` in the file on both sides — identical, and the diff touches none of them. Chasing the missing 31st would be chasing a number that was never there.
  - *Also corrected.* The diff has **three** hunks, not two. `SMOOTH_JOIN_RELATIVE` is part of Task 2's stated public interface, so it joins the `pub use curves::{…}` re-export — the same one-line class of change Task 1 made for `Edge`. Its doc comment mentions `CLOSURE_SNAP_RELATIVE` without an intra-doc link, deliberately: that constant is private, and linking a public item's docs to it fails the rustdoc gate under `-D warnings`.
- [ ] proptest: `lift(b).flatten(t)` bit-identical, zero panics across all generators
- [ ] v1 fixture migrates; migrated outline flattens bit-identical to the v1 boundary
- [ ] Save → load → `cut_boundary` yields identical points
- [ ] `Join::Smooth` with non-collinear handles is refused; line-to-cubic smooth is accepted
- [ ] `Project::default()` produces a valid tolerance
- [ ] No `PatternBoundary` appears in any serialized `.patal`
- [ ] **`patal-export` compiles and its golden PDF is re-blessed deliberately, not incidentally** (R1)
- [ ] **`total_perimeter_mm` is fallible and every call site handles the error** (R2)
- [ ] **A v1 file carrying `outline`, and a v2 file carrying `boundary`, are both refused by name** (R6)

---

## Self-review

Run against the blueprint with fresh eyes, per the writing-plans skill.

**Spec coverage.** Every §3.x has a task: §3.1→Task 3, §3.2→Tasks 1–2, §3.3→Task 4,
§3.4→Task 5, §3.5→Task 6, §3.6→Task 7, §3.7→Task 8, §3.8→Task 9, §3.9→Task 10,
§3.10→threaded through Tasks 1–8 as TDD steps plus the fixtures in Task 8, §3.11→Task 11,
§3.12→Task 12. §5.1–§5.5's validation lists are covered by the named tests in each task.
§6's five CRITIC add-backs all appear: the `Default` trap (Task 6, with its own test), the
pure migration (Task 8), `PieceId`'s reader (Task 5), directional grain (Task 4), and the
validated `Smooth` (Task 2).

**Gaps found and closed while writing.** Three, all recorded as R1/R2/R6 with tasks
attached. R1 is the significant one — an entire crate in the blast radius that the
blueprint's touch-map does not list, because it did not exist yet.

**Type consistency.** `SeamPath::edges()` is used under that name from Task 1 onward;
`Edge::geometry()` returns `EdgeSegment` by value (it is `Copy`) everywhere; `cut_boundary`
takes `tolerance_mm: f64` on the piece and nothing on the project from Task 7 onward;
`PatternPiece::from_boundary` is defined in Task 7 and consumed in Task 8's migration.
`GrainLine::new` returns `Result<_, PatternError>` and is consumed as such.

**One thing deliberately left unresolved.** The blueprint's watch item on an upper bound for
flatten tolerance. A tolerance of 1e9 turns every curve into a straight line — useless, but
not *wrong* in the correct-or-loud sense, and inventing a ceiling means inventing a number.
Revisit if a real user ever sets one; do not guess a limit now.

**What this plan does not make progress on, stated so a green wave does not read as more
than it is.** The PDF has still never been printed. No pattern maker has been found. Neither
is code, and neither moves because of anything above.

