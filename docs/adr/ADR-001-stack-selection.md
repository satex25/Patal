---
id: ADR-001
title: Stack Selection — Target 1
status: accepted
date: 2026-08-07
tags: [adr, architecture, stack]
---

# ADR-001 — Stack Selection (Target 1)

## Status
**Accepted** — 2026-08-07

## Context
Two delivery targets exist for Pātāl:

- **Target 1 (active):** App Store–grade native app — iOS / iPadOS / macOS off one core.
- **Target 2 (deferred):** universal Windows + macOS desktop, heavier feature set.

Target 1 is the sole focus until declared complete.

Stated priorities: world-class polish, high and stable frame rate, native feel,
scalable code. Rust was specified for the heavy-lifting core.

Tailwind CSS and C++ were both proposed for the frontend.

## Decision

### Core — Rust
All domain logic, persistence, sync, and compute live in a Rust crate compiled to a
static library and packaged as an `.xcframework` (arm64 iOS, arm64 + x86_64 macOS).

### UI — SwiftUI
Native SwiftUI shell. Chosen for gesture handling, ProMotion 120Hz, accessibility,
and system integration — none of which cross-platform UI layers match.

### Graphics / animation — Metal
Custom shader-, particle-, and timeline-driven surfaces render through Metal,
embedded in SwiftUI. SwiftUI `Canvas` covers mid-tier work not worth a full pipeline.

### Bridge — UniFFI or swift-bridge
Deferred pending data-shape analysis. UniFFI favours maintainability;
swift-bridge favours a hot boundary.

## Rejected

### Tailwind CSS — rejected
Tailwind is a CSS utility framework, not a language. It requires an HTML/DOM
rendering context and cannot style SwiftUI or Metal. Adopting it would have meant
routing the UI through WKWebView, capping scroll physics and gesture responsiveness
below the stated quality bar, and inviting App Store Guideline 4.2 scrutiny.

**Accepted cost:** frontend work is Swift. No Tailwind anywhere in Target 1.

### C++ — rejected
Not a frontend technology on Apple platforms, and redundant beside Rust in the core.
Including it would add a second FFI boundary for no benefit.

### Path B — Rust-all-the-way (Dioxus 0.7 / Tauri v2) — rejected
Genuinely viable: real Tailwind support, one codebase covering iOS + macOS + Windows,
making Target 2 nearly free. Rejected because its ceiling on native feel sits below
the Target 1 quality bar. Revisit only if Target 1 priorities change.

## Consequences

- **Positive:** highest achievable polish ceiling on Apple platforms. Rust core
  recompiles unchanged for Windows, so Target 2's backend is complete on day one.
- **Negative:** Target 2 requires a full UI rewrite. Swift is a hard dependency.
- **Constraint — FFI boundary:** the render loop must **never** cross FFI per frame.
  Rust hands over batched buffers or shared memory; Metal reads them directly.
  Chatty FFI at 120Hz consumes the entire frame budget and cannot be recovered
  through shader tuning. The boundary is designed around this from the start.
- **Constraint — core purity:** the Rust core imports no platform UI types and
  remains free of Apple-specific assumptions.

## Resolved 2026-08-12

This ADR previously carried an open item: *"Domain of the application — not yet
specified. Module layout and bridge choice remain blocked on it."* Both are settled,
and neither is blocked.

**Domain: garment pattern CAD.** Drawing pattern pieces, applying seam allowances,
and taking them to a cutting line. `docs/memorandum.md` is the long form.

**Module layout**, following from that domain — four crates, split by what each one
is allowed to be wrong about:

| Crate | Responsibility |
|---|---|
| `patal-geometry` | The cut path. `Point2`, `PatternBoundary`, `offset`, and the `SeamPath` curve layer above it. |
| `patal-materials` | `Material`, `MaterialId`, `MaterialLibrary`. |
| `patal-pattern` | `PatternPiece`, `Project`, `Measurement`, `Document`. |
| `patal-ffi` | The uniffi surface. |

**Bridge: uniffi**, already in use by `patal-ffi` at 0.28. The ADR had deferred this
between uniffi and swift-bridge "pending data-shape analysis"; the data shape is now
known, and it is document-and-user-action shaped rather than hot-loop shaped. Document
operations run at user-action frequency, so uniffi's maintainability wins and
swift-bridge's hot-boundary advantage buys nothing that the per-frame constraint below
does not already forbid.

## Where the FFI constraint stands

The per-frame rule is the one constraint here most likely to be violated by accident,
because nothing fails when you do — it just costs the frame budget. It has since been
measured from the Rust side: a full `flatten → offset → self-intersection check` on a
four-cubic bodice front costs about 88µs at manufacturing tolerance, roughly 1% of a
120Hz frame (`engine/crates/geometry/benches/drag_loop.rs`).

That result does **not** relax this rule. It says the geometry is cheap; it says
nothing about the boundary. A per-frame call through uniffi would still violate this
constraint at 120Hz regardless of how little work happens on the other side.

## Note on Tauri

Path B was rejected above as a *shipping* target and remains rejected. ADR-005 unfreezes
the Tauri app as a non-shipping engineering harness, which is a different question and
does not reopen this one.
