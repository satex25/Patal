# Pātāl

CADS PATAL
Computer Aided Design System For Patterns

Pātāl (पाताल) — in Hindu cosmology, the netherworld: one of the seven realms
beneath the earth, vast and richly structured, built downward from a surface
few ever see. Formerly named *Patruin* (Irish "patrún," pattern); renamed
2026-08-07 — see [ADR-002](docs/adr/ADR-002-naming-convention.md).

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
│       ├── geometry/       patal-geometry  — Point2, PatternBoundary, offset, SeamPath curves
│       ├── materials/      patal-materials — Material, MaterialLibrary
│       ├── pattern/        patal-pattern   — PatternPiece, Project, measurements
│       └── ffi/            patal-ffi       — uniffi bindings exposed to Swift
├── apps/
│   ├── native/              SwiftUI — iPhone, iPad, Mac (one shared codebase)
│   └── desktop/              Tauri — engineering harness, NOT a shipping target (ADR-005)
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
  is no silent corruption path left in this crate. A failed offset says
  which two edges cross, so a UI can point at the problem rather than only
  naming it. `PatternPiece` and `Project` sit on top with the same
  discipline: `seam_allowance_mm` is validated, not a bare public field.
  Curves live in a layer *above* the polygon kernel — `SeamPath` and
  `EdgeSegment` are authored, `flatten` discretizes them, and the kernel is
  untouched ([ADR-003](docs/adr/ADR-003-curve-representation.md)). Materials
  have stable identity and a project carries a document schema version
  ([ADR-004](docs/adr/ADR-004-document-format.md)). 89 unit tests plus a
  property suite and a closed-form curve oracle, `cargo clippy --workspace
  --all-targets -- -D warnings` clean, `cargo deny` clean on all four checks.
- `patal-ffi`: exports the engine's fallible boundary operations
  (perimeter, offset) across the uniffi boundary as `Result`, not as NaN —
  a caller on the other side gets a real error, not a number it has to
  guess is wrong. This is verified by Rust-side round-trip tests only.
  **No Swift bindings are generated or committed**, there is no XCFramework,
  and nothing in `apps/native` calls into this crate yet — the seam exists
  and is tested from the Rust side, but it is not yet a working pipeline.
- `apps/native`: a Swift package (`PatalKit`) with hand-written model
  mirrors of the engine's domain types plus a basic SwiftUI shell. It holds
  `Point2`, `PatternBoundary` (construction invariant and `Codable` matching
  the Rust engine's bare-point-array wire shape exactly), `Material`,
  `PatternPiece`, and `Project`. It deliberately holds **no geometry**: the
  368-line hand-ported offset kernel that used to live here was deleted, so
  there is exactly one implementation of the math that decides where cloth
  gets cut. See the note below. Never built or tested in this environment —
  there is no macOS toolchain here, and CI's `native` job is the only
  `swift build` this code has ever had.
- `apps/desktop`: **an engineering harness, not a product.**
  [ADR-001](docs/adr/ADR-001-stack-selection.md) rejected Tauri as a shipping
  target and that stands; [ADR-005](docs/adr/ADR-005-tauri-as-engineering-harness.md)
  explains why it is unfrozen for development anyway. It links the engine
  crates directly with no FFI boundary, and it is the only thing in this repo
  that runs on the Windows machine Pātāl is developed on. It draws a bodice
  front with live tolerance and seam-allowance sliders, reports per-frame cost
  against a 120Hz budget, shows the engine's refusals verbatim when an
  allowance exceeds what the curvature can give, and writes a real `.patal`
  file and reads it back. Disposable by design.

**There is now one implementation of the cut path, not two.** `PatalKit`
used to carry a hand-ported copy of the offset kernel — same mitre limit,
same bevel joins, same winding and self-intersection checks — which meant
two independent implementations decided where cloth gets cut and nothing
checked them against each other. That is a liability rather than a feature:
whichever one drifts, a designer finds out in cloth. It was deleted rather
than pinned in place with a cross-language conformance corpus, because
nothing depended on it. There is no Xcode project in this repo, and the
port's only caller was its own test suite. Seam-allowance geometry belongs
to `patal-geometry` and will reach Swift through uniffi-generated bindings.

The remaining Swift/Rust gap is the identity model, and it is unchanged:
Swift's `PatternPiece` and `Project` carry a `UUID` that Rust's types have
no counterpart for, so `PatternPiece`'s `Codable` conformance is
Swift-to-Swift only — unlike `PatternBoundary`'s, which matches the Rust
wire format exactly.

What's deliberately not started: the parametric propagation/constraint
solver (patterns as "a living system" where edits propagate), **export**
(DXF-AAMA/ASTM, tiled PDF at true scale), **grading**, the AI collaborator
layer, and any visual identity (colors/type).

Export and grading deserve a sentence rather than a bullet, because they are
the two capabilities that make this a pattern CAD application and neither is
in any current plan. Both are pure Rust, both run on Windows with no Mac, and
both are testable headlessly. Export is also the cheapest route to real
validation there is: print a tiled PDF at true scale and hand it to a pattern
maker.

Persistence exists as a *format*, not as file I/O. Every domain type derives
`Serialize`/`Deserialize`, `Document` carries a `schema_version`, and
material references are checked on load. What the engine does not do is touch
the disk — no atomic write, no save/load API. The harness does that today, in
disposable code.

## Getting started

### Prerequisites

- **Rust** — the exact toolchain is pinned by `rust-toolchain.toml`; rustup
  picks it up automatically the first time `cargo` runs in this checkout.
- **On Windows: Visual Studio Build Tools** with the "Desktop development
  with C++" workload. Rust's MSVC target links with `link.exe`, which ships
  with that workload and nothing else.
- **Node 20+** for the desktop app (`apps/desktop/.nvmrc` pins the version).
- **macOS with full Xcode** for `apps/native` — the Command Line Tools alone
  can `swift build` but cannot `swift test`, because `XCTest` ships with
  Xcode proper.

### Building

```sh
# Engine
cd engine && cargo test --workspace

# Native app (Swift package only, until Xcode wires it into a real app — see apps/native/README.md)
cd apps/native && swift build

# Desktop app
cd apps/desktop && npm install && npm run tauri dev
```

### If you are on Windows and use Git Bash

`cargo build` will fail with something that looks nothing like the real
problem:

```
= note: /usr/bin/link: extra operand '/NOLOGO'
error: linking with `link.exe` failed: exit code: 1
```

Git Bash ships a coreutils `link` that shadows MSVC's `link.exe` on `PATH`,
so cargo invokes the wrong program. **rustc's own hint is misleading here:**
it suggests repairing your Visual Studio installation, which is fine and is
not the problem.

Use the committed wrapper instead — it locates the toolset with `vswhere`,
sources `vcvars64.bat`, and runs cargo with the right `PATH`:

```sh
cmd //c 'scripts\cargo.bat test --workspace --locked'
cmd //c 'scripts\cargo.bat clippy --workspace --all-targets --locked -- -D warnings'
cmd //c 'scripts\cargo.bat fmt --check'
```

From PowerShell or `cmd`, drop the `cmd //c` and call `scripts\cargo.bat`
directly. It defaults to the `engine/` workspace; set `PATAL_CARGO_DIR` to
point it elsewhere:

```sh
PATAL_CARGO_DIR='C:\path\to\patal\apps\desktop\src-tauri' cmd //c 'scripts\cargo.bat clippy'
```

This deliberately stays out of `.cargo/config.toml`: the vcvars path is
machine-local and would break CI, which already has a working linker.

A "Developer Command Prompt for VS" also works and needs no wrapper — the
wrapper exists so that the ordinary shell people already have open does the
right thing.

## License

Proprietary — see [`LICENSE`](LICENSE). The source is public for reference;
that is not a grant of any right to use it.
