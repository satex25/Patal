# Contributing to Pātāl

Pātāl is proprietary (see [`LICENSE`](LICENSE)); the source is public for
reference. This file is here because "future me, six months from now" is the
contributor this project actually has, and that person needs the same
onboarding a stranger would.

## The one rule that outranks the others

From the header of `engine/crates/geometry/src/lib.rs`:

> Every operation here is either correct or loud. A pattern piece that is
> silently wrong is worse than one that refuses to compute: the first gets
> cut out of cloth, the second gets fixed.

Concretely, in anything that touches geometry:

- A fallible operation returns `Result`, never a fallback number. No
  `unwrap_or(0.0)`, no clamping a NaN to something plausible.
- An error names the *actual* failure. Do not collapse several causes into
  one convenient variant — `offset()` used to report every construction
  failure as `OffsetCollapsed`, which sent callers after the wrong problem.
- Errors carry enough to act on. `OffsetSelfIntersects` names which two
  edges cross, because a UI that can only say "this is wrong somewhere"
  cannot help a designer fix it.
- Tolerances are explicit parameters, never silently defaulted.

## Setup

See [Getting started](README.md#getting-started) in the README for
prerequisites and, if you are on Windows and use Git Bash, the `link.exe`
shadowing gotcha and the `scripts/cargo.bat` wrapper that resolves it. That
wrapper exists because the error rustc prints for this points at the wrong
cause.

## Before you push

The full gate, which is what CI runs:

```sh
cmd //c 'scripts\cargo.bat fmt --check'
cmd //c 'scripts\cargo.bat clippy --workspace --all-targets --locked -- -D warnings'
cmd //c 'scripts\cargo.bat test --workspace --locked'
cmd //c 'scripts\cargo.bat deny check licenses bans sources'
RUSTDOCFLAGS='-D warnings' cmd //c 'scripts\cargo.bat doc --no-deps --workspace --locked'
```

(Drop `cmd //c` outside Git Bash. On macOS and Linux, plain `cargo` in
`engine/` works — the wrapper is a Windows linker workaround, nothing more.)

Two things that are easy to forget:

- **`apps/desktop` is a compile gate.** Its Tauri backend links
  `patal-geometry` and `patal-pattern` directly and is checked under
  `-D warnings` in CI. "Frozen" describes the product roadmap, not the build
  graph. If you change a domain type's shape, check it:
  `PATAL_CARGO_DIR='...\apps\desktop\src-tauri' cmd //c 'scripts\cargo.bat clippy --all-targets --locked -- -D warnings'`
- **`apps/native` cannot be built on Windows.** CI's `native` job on
  `macos-latest` is the only verification Swift code gets. Assume any Swift
  edit is unverified until that job is green.

`cargo deny check advisories` is deliberately *not* in the blocking set — it
runs weekly on a schedule instead. Run it when you touch dependencies.

The property suite in `engine/crates/geometry/tests/properties.rs` runs 256
cases per property by default. Before merging anything that touches the
kernel, give it a real sweep:

```sh
PROPTEST_CASES=100000 cmd //c 'scripts\cargo.bat test --locked -p patal-geometry --test properties'
```

Export it from your shell rather than using `set VAR=x && ...` inside
`cmd //c` — that form does not reach the test process, and the suite will
silently run 256 cases while looking like it ran 100,000.

## Architectural constraints

These are recorded as ADRs in [`docs/adr/`](docs/adr/) and are not
re-litigated in review comments:

| | Constraint | Where |
|---|---|---|
| C1 | Correct or loud. | geometry crate header |
| C2 | `#![forbid(unsafe_code)]` stays in every crate. | all four crates |
| C3 | The engine imports no platform UI types and carries no Apple assumptions. | ADR-001 |
| C4 | The render loop never crosses FFI per frame. Rust hands over batched buffers. | ADR-001 |
| C5 | `Pātāl` in prose and UI; `Patal` in anything a toolchain touches. | ADR-002 |
| C6 | `PatternBoundary` invariants live in the constructor. Private field, serde routed through `try_from`. | geometry crate |
| C7 | `PatternBoundary`'s wire format is a bare `Vec<Point2>`. | geometry + `Geometry.swift` |

C4 is the one most likely to be violated by accident, because nothing fails
when you do. When adding anything to `patal-ffi`, ask explicitly: *could a UI
call this once per frame?* If yes, batch it.

## Geometry, specifically

There is exactly one implementation of the math that decides where cloth
gets cut, and it is `patal-geometry`. Swift used to carry a second one; it
was deleted rather than pinned in place. Do not reintroduce one in any
language — the failure mode of two implementations drifting is not a red
test, it is a garment cut wrong.

The kernel (`PatternBoundary` and `offset`) is deliberately stable. Curves
are built as an authoring layer *on top* of it that flattens to a
`PatternBoundary`, not by teaching the kernel about curves. All of the
kernel's original tests must keep passing unmodified; that is the evidence
the cut path was not disturbed.

## Commits

Explain why, not what — the diff already says what. A commit that fixes
something subtle should leave behind the reasoning that made it subtle, so
the next person does not undo it. If a change is based on something you
verified by running it, say what you ran.
