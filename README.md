# Patruin

Patruin — the modern Irish word for patterns (patrún). 

A professional garment pattern creation platform: from idea to production-ready pattern in
one workspace, across iPhone, iPad, Mac, and Windows.

- Repo: [github.com/satex25/patruin](https://github.com/satex25/patruin)
- Website / downloads: [satex25.co](https://satex25.co)

The full founding memorandum is in [`docs/memorandum.md`](docs/memorandum.md).
Everything below is the technical shape it maps to.

## Architecture

One Rust engine, two front ends:

```
patruin/
├── engine/                 Rust workspace — platform-agnostic core
│   └── crates/
│       ├── geometry/       patruin-geometry  — Point2, PatternBoundary, seam-allowance offset
│       ├── materials/      patruin-materials — Material, MaterialLibrary
│       ├── pattern/        patruin-pattern   — PatternPiece, Project, measurements
│       └── ffi/            patruin-ffi       — uniffi bindings exposed to Swift
├── apps/
│   ├── native/              SwiftUI — iPhone, iPad, Mac (one shared codebase)
│   └── desktop/              Tauri (Rust + Tailwind) — Windows build for satex25.co
└── docs/
    └── memorandum.md
```

**Why this split:** the memorandum's "Pattern Engine" and "Material System"
are the platform-independent core — the same geometry math and material
model must produce identical results whether a designer is on an iPad or a
Windows laptop. `engine/` is that core. `patruin-ffi` is the seam: the Tauri
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

This is a foundation, not a product yet. What's real today:

- `engine/`: real geometry (polygon perimeter + outward seam-allowance
  offset, orientation-independent), a material model matching every
  attribute the memorandum lists, and a `Project`/`PatternPiece` layer.
  15 unit tests passing (`cargo test --workspace` inside `engine/`).
- `patruin-ffi`: exports a first slice of the engine (boundary perimeter and
  offset) via uniffi. Swift bindings have been generated and verified to
  parse correctly — the FFI pipeline is proven, not theoretical.
- `apps/native`: a Swift package (`PatruinKit`) with hand-written Swift
  mirrors of the engine's domain types plus a basic SwiftUI shell. Builds
  clean via `swift build`. **Not yet an Xcode project** — see
  `apps/native/README.md` for the one-time setup once Xcode is installed
  (only Command Line Tools are present in this environment).
- `apps/desktop`: a Tauri app whose Rust backend links `patruin-geometry`
  and `patruin-pattern` directly and exposes one command
  (`engine_demo_perimeter_mm`) that a Tailwind-styled placeholder screen
  calls, proving the desktop shell reaches the real engine.

What's deliberately not started: the parametric propagation/constraint
solver (patterns as "a living system" where edits propagate), manufacturing
export (DXF/AAMA, PDF), the AI collaborator layer, and any visual identity
(colors/type) — none of that was specified yet.

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
