# Pātāl

Pātāl (पाताल) — in Hindu cosmology, the netherworld: one of the seven realms
beneath the earth, vast and richly structured, built downward from a surface
few ever see. Formerly named *Patruin* (Irish "patrún," pattern); renamed
2026-08-07 — see `01 Architecture/Decisions/ADR-002 Naming Convention.md`.

A professional garment pattern creation platform: from idea to production-ready pattern in
one workspace, across iPhone, iPad, Mac, and Windows.

- Repo: [github.com/satex25/patal](https://github.com/satex25/patal)
- Website / downloads: [satex25.co](https://satex25.co)

The full founding memorandum is in [`docs/memorandum.md`](docs/memorandum.md).
Everything below is the technical shape it maps to.

## Architecture

One Rust engine, two front ends:

```
patal/
├── engine/                 Rust workspace — platform-agnostic core
│   └── crates/
│       ├── geometry/       patal-geometry  — Point2, PatternBoundary, seam-allowance offset
│       ├── materials/      patal-materials — Material, MaterialLibrary
│       ├── pattern/        patal-pattern   — PatternPiece, Project, measurements
│       └── ffi/            patal-ffi       — uniffi bindings exposed to Swift
├── apps/
│   ├── native/              SwiftUI — iPhone, iPad, Mac (one shared codebase)
│   └── desktop/              Tauri (Rust + Tailwind) — Windows build for satex25.co
└── docs/
    └── memorandum.md
```

**Why this split:** the memorandum's "Pattern Engine" and "Material System"
are the platform-independent core — the same geometry math and material
model must produce identical results whether a designer is on an iPad or a
Windows laptop. `engine/` is that core. `patal-ffi` is the seam: the Tauri
desktop app links the engine crates directly (both are Rust), while the
Swift app will link them through uniffi-generated bindings, packaged as an
XCFramework, once Xcode is available to build that framework.

## Where each memorandum pillar lives

| Memorandum pillar   | Lives in |
|---------------------|----------|
| Pattern Engine       | `engine/crates/geometry`, `engine/crates/pattern` |
| Material System       | `engine/crates/materials` |
| Design Environment    | `apps/native` (SwiftUI), `apps/desktop` (Tailwind) |
| Platform Goals (iPhone/iPad/Mac/Windows) | `apps/native` covers the first three; `apps/desktop` covers Windows |
| Intelligence           | Not yet started — deliberately deferred until the engine and design environment are solid enough to have something for an AI collaborator to act on |

## Status

This is a foundation, not a product yet. What's real today, and what isn't:

- `engine/`: real geometry — polygon perimeter, winding, and an outward/inward
  seam-allowance offset with a mitre limit, self-intersection detection, and
  `hypot`-based distance math. Every fallible input (non-finite coordinates,
  a zero-length edge, an inset larger than the piece can give) returns a
  typed `GeometryError` rather than a plausible-looking wrong number — there
  is no silent corruption path left in this crate. `PatternPiece` and
  `Project` sit on top with the same discipline: `seam_allowance_mm` is
  validated, not a bare public field. 33 unit tests passing
  (`cargo test --workspace` inside `engine/`), `cargo clippy --workspace
  --all-targets -- -D warnings` clean.
- `patal-ffi`: exports the engine's fallible boundary operations
  (perimeter, offset) across the uniffi boundary as `Result`, not as NaN —
  a caller on the other side gets a real error, not a number it has to
  guess is wrong. This is verified by Rust-side round-trip tests only.
  **No Swift bindings are generated or committed**, there is no XCFramework,
  and nothing in `apps/native` calls into this crate yet — the seam exists
  and is tested from the Rust side, but it is not yet a working pipeline.
- `apps/native`: a Swift package (`PatalKit`) with hand-written Swift
  mirrors of the engine's domain types plus a basic SwiftUI shell. As of
  this port, `PatternBoundary.offset` and `PatternPiece`'s validated seam
  allowance are ported line-for-line from the Rust engine — same mitre
  limit, same bevel-join and self-intersection checks, same errors thrown
  instead of a wrong-looking number — so all three Apple platforms can now
  compute a real seam allowance, not just Windows. `PatternBoundary` also
  gained hand-written `Codable` matching the Rust engine's JSON wire shape
  (a bare point array) exactly. Builds clean via `swift build`; `swift
  test` needs full Xcode for `XCTest` and cannot run in this environment
  (only Command Line Tools are present), so the port was instead verified
  by running every one of the Rust engine's own numeric test cases —
  including the specific inputs that used to corrupt the old kernel —
  through a throwaway executable and confirming the outputs match to six
  decimal places; see `apps/native/README.md` for the Xcode project setup
  needed to actually run `swift test` here. This is still a second,
  independent implementation of the domain model, not a binding to the
  first — see the note on that below.
- `apps/desktop`: a Tauri app whose Rust backend links `patal-geometry`
  and `patal-pattern` directly (no FFI boundary — both are Rust) and
  exposes one command, `engine_demo_perimeter_mm`, that a Tailwind-styled
  placeholder screen calls and displays or reports as an error. This does
  demonstrate the desktop shell reaching the real, hardened engine; it does
  not yet exercise the engine's harder paths (offset, validation failures).

**The Swift mirror is duplicated, not derived, and that is real architectural
debt — narrower now, but not closed.** `PatalKit`'s types are hand-written
to look like the Rust engine's, not generated from it, so the two can still
drift out of sync on the next change to either side; porting `offset` by
hand fixed today's gap but didn't fix the mechanism that created it. The
identity model has also diverged and stayed diverged: Swift's `PatternPiece`
and `Project` carry a `UUID` that Rust's types have no counterpart for, so
`PatternPiece`'s `Codable` conformance is Swift-to-Swift only — it does not
yet match the Rust engine's wire format the way `PatternBoundary`'s does.
The long-term fix is wiring `apps/native` to the real engine through uniffi,
not maintaining two implementations in parallel; that work has not started.

What's deliberately not started: the parametric propagation/constraint
solver (patterns as "a living system" where edits propagate), a
serialization layer for saving/loading a `Project` (nothing in `engine/`
derives `Serialize`/`Deserialize` yet, so no document can currently leave
process memory), manufacturing export (DXF/AAMA, PDF), the AI collaborator
layer, and any visual identity (colors/type) — none of that was specified
yet.

## Getting started

```sh
# Engine
cd engine && cargo test --workspace

# Native app (Swift package only, until Xcode wires it into a real app — see apps/native/README.md)
cd apps/native && swift build

# Desktop app
cd apps/desktop && npm install && npm run tauri dev
```

## License

Proprietary — see [`LICENSE`](LICENSE). The source is public for reference;
that is not a grant of any right to use it.
