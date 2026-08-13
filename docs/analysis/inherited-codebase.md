---
title: Inherited Codebase — Full Analysis
date: 2026-08-07
status: verified
tags: [codebase, audit, patruin, migration]
---

# Inherited Codebase — Full Analysis

> **Relocated 2026-08-07:** this audit was performed at `Desktop\Pātāl\Patal-main\Patal-main\`,
> which no longer exists — the code now lives at `C:\Users\User\patal\` (git-initialized,
> baseline commit `8d3a447`; see [2026-08-07 rename and relocate](../plans/2026-08-07-rename-and-relocate.md)). Paths below are historical.
>
> **Read every `patruin-*` below as `patal-*`.** This is a pre-rename snapshot and the
> old names are left intact deliberately — rewriting them would make it a worse record
> of what was actually found. The mapping table near the end of this note is the
> authoritative old-to-new list; [the ADR index](../adr/README.md) points at ADR-002,
> which fixes the naming rule itself.
>
> **The state described here is superseded.** Test counts, crate contents and the
> "what is missing" list all moved on during the 2026-08-12 hardening wave — curves,
> material identity, a document schema version and a property suite all landed after
> this was written. See [status](../status.md) for the current picture and [roadmap](../roadmap.md) for what
> is still genuinely absent.

Source: `Desktop\Pātāl\Patal-main\Patal-main\`
Old name: **Patruin**. New name: **Pātāl** / `Patal` (see [ADR-002](../adr/ADR-002-naming-convention.md)).

> Every claim below marked ✅ was **executed**, not read. Rust 1.97.1 was installed
> in the analysis sandbox and the workspace was compiled and tested for real.

---

## 1. What this product is

**A professional garment pattern creation platform.** Not a sewing app — a CAD
system for clothing. The founding memorandum (`docs/memorandum.md`, v1.0) states
the mission as *"the world's most intuitive professional garment pattern creation
platform"*, taking a designer from idea → production-ready pattern in one workspace.

The name *Patrúin* is Irish for "patterns" (plural of *patrún*).

### The seven pillars from the memorandum

| Pillar | Intent |
|---|---|
| **Pattern Engine** | Converts creative intent into mathematically accurate construction geometry. Patterns are a *"living system of interconnected relationships"* — edits propagate. |
| **Material System** | Materials carry physical properties (weight, drape, stretch, rigidity) that **actively inform construction**, not just appearance. |
| **Design Environment** | Minimal visual noise, smooth animation, non-destructive workflows, professional polish. |
| **Intelligence** | AI as a collaborative design partner. Creator retains ownership. |
| **Platform Goals** | iPhone, iPad, Mac, Windows — projects move seamlessly between devices. |
| **Creative Philosophy** | Software should feel invisible. Every decision editable, interconnected, reversible. |
| **Guiding Principles** | Creativity · Precision · Accessibility · Flexibility · Elegance · Performance · Craftsmanship |

**Development test** (from the memorandum, verbatim):
> Does this make it easier for someone to transform an idea into a beautifully
> engineered, production-ready garment while expanding their creative possibilities?

---

## 2. Architecture as built

```
Patal-main/
├── engine/                    Rust workspace — platform-agnostic core
│   └── crates/
│       ├── geometry/          patruin-geometry   754 LOC
│       ├── materials/         patruin-materials  213 LOC
│       ├── pattern/           patruin-pattern    343 LOC
│       └── ffi/               patruin-ffi        142 LOC  (uniffi 0.28)
├── apps/
│   ├── native/                SwiftUI — iPhone/iPad/Mac  (588 LOC Swift)
│   └── desktop/               Tauri + Tailwind — Windows  (51 LOC)
├── docs/memorandum.md
├── rust-toolchain.toml        pinned 1.97.1
└── .github/workflows/ci.yml   3 jobs: engine, desktop, native
```

Total ~2,100 LOC. 633 KB. Small, but the quality density is unusually high.

**Dependency direction is clean:** `geometry` ← `pattern` → `materials`, and `ffi`
sits on top of all three. No domain crate knows about FFI, UI, or a platform.
`#![forbid(unsafe_code)]` on geometry, pattern, and ffi.

---

## 3. Verified state ✅

Compiled and ran `cargo test --workspace` on Rust 1.97.1:

| Crate | Tests | Result |
|---|---|---|
| patruin-geometry | 25 | ✅ pass |
| patruin-pattern | 10 | ✅ pass |
| patruin-materials | 8 | ✅ pass |
| patruin-ffi | 4 | ✅ pass |
| **Total** | **47** | **✅ all pass** |

Swift side: 16 XCTest cases written, **never executed** (needs full Xcode).

### The engine is genuinely good

This is not scaffold code. `patruin-geometry` implements a real seam-allowance
offset with engineering care that is rare:

- **`hypot` for distance** — avoids overflow on large coordinates and underflow on small.
- **Mitre limit (4.0)** — without it, offset length grows as `1/sin(θ/2)`, so a 6°
  collar point would throw a vertex hundreds of mm off the piece. Falls back to a bevel.
- **Relative area epsilon** — `perimeter² × 1e-12`, not an absolute constant. An
  absolute epsilon calls a metre-scale piece degenerate and a mm-scale one fine.
- **Normalised parallelism test** — `|sin θ|` compared after dividing by edge lengths,
  so the parallel decision depends on angle, not on edge length.
- **Edge-direction reversal check** — the clever one. When an inset eats more than the
  piece has to give, every edge reverses, which is a 180° rotation that *preserves both
  winding and area sign*. A winding check would pass a 20 mm square "inset" by 15 mm as
  a plausible 10 mm square. This code checks each offset edge still runs the same
  direction as its source. That is a subtle bug caught properly.
- **Typed errors, never NaN** — 7 `GeometryError` variants. The stated principle:
  *"A pattern piece that is silently wrong is worse than one that refuses to compute:
  the first gets cut out of cloth, the second gets fixed."*
- **Invariants enforced by construction** — `PatternBoundary.points` is private, and
  `#[serde(try_from = "Vec<Point2>")]` routes deserialization through the validating
  constructor so a file on disk cannot smuggle in an invalid boundary.

Same discipline in `pattern`: `seam_allowance_mm` is private behind a validating
setter, and `PatternPiece` deserializes through a DTO that re-runs validation.

---

## 4. Defects and drift found

### 4.1 🔴 Material JSON interop is broken — **proven, not inferred**

`patruin-materials` carries this doc comment:

> *"matching PatruinKit's Swift `Drape` raw values exactly, so a JSON document
> produced by either mirror deserializes cleanly in the other."*

**This is false.** Executed test — feeding Swift-shaped `Material` JSON to the Rust type:

```
Swift -> Rust : Err(Error("missing field `surface_texture`", line: 11, column: 5))
```

Three independent causes:

1. **Key casing.** Rust emits `weight_gsm`, `thickness_mm`, `surface_texture`,
   `durability_notes`. Swift's default `Codable` emits `weightGSM`, `thicknessMM`,
   `surfaceTexture`, `durabilityNotes`. Rust `Material` has **no** `#[serde(rename_all)]`
   — only the `Drape`/`Rigidity` enums do. So the claim is true of the enums and
   false of the struct containing them.
2. **Four fields missing entirely from Swift.** Rust requires `layer_compatibility`,
   `stitch_recommendations`, `reinforcement_requirements`, `manufacturing_considerations`
   (`Vec<String>`, no `#[serde(default)]`). Swift's `Material` has none of them.
   These are *memorandum pillar* fields — the properties that make materials
   "actively inform construction."
3. **Swift has an extra `id: UUID`** with no Rust counterpart.

Rust actually emits:
```json
{"name":"Cotton Twill","weight_gsm":280.0,"thickness_mm":null,"stretch_percent":null,
 "drape":"structured","rigidity":"medium","surface_texture":"","durability_notes":"",
 "layer_compatibility":[],"stitch_recommendations":[],"reinforcement_requirements":[],
 "manufacturing_considerations":[]}
```

The README documents the identity/`UUID` divergence but **not** this. It is worse
than advertised.

### 4.2 🔴 The Swift mirror is a second implementation, not a binding

~540 lines of Swift hand-reimplement the Rust engine — including the entire offset
algorithm, mitre limit, bevel joins, winding, and self-intersection checks. Two
implementations of safety-critical geometry that must be kept identical by hand.

The README is honest about this ("real architectural debt — narrower now, but not
closed"). It is still the single largest risk in the codebase: a fix applied to one
side and not the other produces a garment cut wrong on one platform only.

**`patruin-ffi` already exists and works.** The bridge is built and tested from the
Rust side. What's missing is generating the Swift bindings and packaging the
XCFramework — which is exactly what `cargo-swift` automates.

### 4.3 🟡 README is stale in two places

- Claims **"33 unit tests passing."** Actual: **47** ✅.
- Claims *"nothing in `engine/` derives `Serialize`/`Deserialize` yet, so no document
  can currently leave process memory."* **False** — `Point2`, `PatternBoundary`,
  `Material`, `MaterialLibrary`, `PatternPiece`, `Project` all derive both, and there
  is a passing test named `project_round_trips_through_json_including_nested_validation`.
  Serialization exists; a *document/file layer* is what's missing.

### 4.4 🟡 Verification of the Swift port is non-reproducible

The Swift `offset` port was validated by a throwaway executable that was then
**deleted**. The evidence no longer exists in the repo. The 16 XCTest cases have
never run. This must be re-verified on the Mac before the Swift math is trusted.

---

## 5. Gaps against the memorandum

### Acknowledged in the README
- Parametric propagation / constraint solver — **the central promise**, not started
- Manufacturing export (DXF/AAMA, PDF)
- AI collaborator layer
- Visual identity (no colors, no type)

### Not acknowledged anywhere — found in this audit

- 🔴 **No curves.** `PatternBoundary` is a straight-edged polygon only. Necklines,
  armholes, sleeve caps, hems, princess seams — the entire vocabulary of garment
  patternmaking is curved. The geometry crate notes curves as "a planned extension"
  and warns lengths "will read slightly short." For a professional pattern tool this
  is not an extension, it is a missing core primitive. Bézier or spline support
  changes the shape of `PatternBoundary`, `offset`, and every consumer.
- 🔴 **No grading.** Scaling a pattern across a size run is fundamental to
  professional patternmaking and appears **nowhere** — not in the code, not in the
  memorandum. A genuine gap in the founding document, not just the implementation.
- 🔴 **No pattern primitives.** Darts, notches, grainlines, seam types, pleats,
  facings, buttonholes have no representation. `PatternPiece` is currently a named
  polygon with a seam allowance and an optional material.
- 🟡 **No canvas or rendering.** `ContentView` is a `NavigationStack` + `List`. There
  is no drawing surface, no pan/zoom, no vertex editing. The entire design environment
  is unbuilt.
- 🟡 **No document layer.** Types serialize, but there is no file format, no save/load,
  no undo stack, no autosave. The `.patruin` extension is referenced in a doc comment
  but does not exist.
- 🟡 **No sync.** "Projects move seamlessly between devices" has no implementation.

---

## 6. How this maps to our decisions

| Our decision | Status against inherited code |
|---|---|
| Rust core | ✅ Already exactly this. Clean, tested, platform-agnostic. |
| SwiftUI for Target 1 | ✅ `apps/native` exists as a Swift package. |
| Metal for graphics | ⚠️ Nothing exists yet. This is where it goes — the pattern canvas. |
| Bridge: UniFFI vs swift-bridge | ✅ **Resolved by existing code — uniffi 0.28.** `cargo-swift` wraps uniffi, so the cloned reference repos line up exactly. ADR-001's open item can close. |
| Target 2 deferred | ✅ `apps/desktop` (Tauri + Tailwind) is already-started Target 2 work. **Freeze it.** |
| Tailwind rejected for Target 1 | ✅ Consistent — Tailwind exists only in `apps/desktop`, which is Target 2. |

**Note:** the earlier Tailwind discussion is now resolved by fact rather than
preference. Tailwind already lives in the codebase, scoped to the Windows Tauri
app. It was never going to touch the Apple app either way.

---

## 7. Rename scope

**124 occurrences of "patruin" / "Patruin" across 26 files.**

Not a find-and-replace — these are load-bearing identifiers:

| Kind | From | To |
|---|---|---|
| Crate names | `patruin-geometry`, `patruin-materials`, `patruin-pattern`, `patruin-ffi` | `patal-geometry`, `patal-materials`, `patal-pattern`, `patal-ffi` |
| Rust module paths | `patruin_geometry::` | `patal_geometry::` |
| Swift module | `PatruinKit` | `PatalKit` |
| Swift test target | `PatruinKitTests` | `PatalKitTests` |
| Swift package | `Patruin` | `Patal` |
| npm package | `patruin-desktop` | `patal-desktop` |
| Tauri identifier | `co.satex25.patruin` | `co.satex25.patal.desktop` — this row's original `com.patal.desktop` proposal was **not** adopted; see ADR-002 |
| Directory | `Sources/PatruinKit/` | `Sources/PatalKit/` |
| Display text | "Patruin" | "Pātāl" |
| Repo / URL | `github.com/satex25/patruin` | `github.com/satex25/patal` |
| File extension | `.patruin` | `.patal` |

Both `Cargo.lock` files must be regenerated, not edited.

**Important:** display strings take the diacritic form **Pātāl**; every identifier,
path, and package name takes **Patal**. The memorandum's Irish etymology no longer
explains the new name — that section needs rewriting, not renaming.

---

## 8. Assessment

**This is a strong foundation with a narrow, deep base and no breadth yet.**

The engine's numerical discipline is better than most shipped CAD code — the
edge-reversal collapse check in particular is the kind of thing that is normally
found by a customer cutting fabric wrong, not by a developer. Whoever wrote this
understood that in this domain, a wrong number becomes ruined cloth.

The honest summary: **roughly 5% of the memorandum is built, and that 5% is built
very well.** What exists is the mathematical substrate. What does not exist is the
product — the canvas, the curves, the propagation solver, the document, the design
environment.

The two things that must be decided before real feature work:

1. **Curves.** Adding them later means rewriting `PatternBoundary`, `offset`, and
   every consumer. Deciding now is far cheaper.
2. **Kill the Swift mirror.** Wire `apps/native` to `patal-ffi` through `cargo-swift`
   and delete ~540 lines of duplicated geometry. Until that happens, every engine
   change costs double and risks platform-divergent output.
