# Pātāl — Project Overview

_Captured 2026-08-13 from the repo at `C:\Users\User\patal` (also `github.com/satex25/patal`, proprietary/source-available)._

## What it is

Pātāl (पाताल — the Hindu netherworld, seven structured realms beneath the surface) is a professional garment pattern CAD platform: idea → silhouette → construction geometry → production-ready pattern, in one workspace across iPhone, iPad, Mac, and Windows. Formerly named _Patruin_; renamed 2026-08-07 (ADR-002).

Mission: the most intuitive professional garment pattern creation platform — a living system where a change (e.g. a bust measurement) propagates through dependent pieces, not a drawing tool with a garment theme.

## Architecture

One Rust engine, two front ends:

- `engine/` — platform-agnostic core (`patal-geometry`, `patal-materials`, `patal-pattern`, `patal-export`, `patal-ffi` for uniffi/Swift bindings)
- `apps/native` — SwiftUI, iPhone/iPad/Mac, shared codebase, links engine via uniffi + XCFramework (not yet built — no Xcode access locally)
- `apps/desktop` — Tauri, links engine crates directly. Explicitly **not a shipping target** (ADR-001), kept alive as a disposable engineering harness (ADR-005) — it's the only thing that runs on the Windows dev machine.
- `docs/` — status.md (source of truth), roadmap.md, memorandum.md (founding vision), adr/, setup/, analysis/, plans/

Hard rules: render loop never crosses FFI per-frame (ADR-001); Metal for rendering, not wgpu; every fallible geometry op returns a typed error, never a silently-wrong number ("correct or loud" — the one rule, from `geometry/src/lib.rs`).

## Status as of 2026-08-13

**Foundation stage, not a product yet.** Real and tested:

- `patal-geometry`: polygon perimeter/winding, seam-allowance offset with mitre limit, self-intersection detection with named crossing edges, curve layer (`SeamPath`/`EdgeSegment`/`flatten`) above the polygon kernel. 89 unit tests + property suite + closed-form curve oracle, clippy-clean, `cargo deny`-clean.
- `patal-ffi`: exports fallible boundary ops as `Result` across uniffi. Rust-side tested only — no Swift bindings generated yet, nothing in `apps/native` calls it yet.
- `apps/native`: Swift package `PatalKit` — model mirrors only, deliberately **no geometry** (a 368-line hand-ported offset kernel was deleted so there's one implementation of the cut line). **First successful `swift build` + `swift test` (12/12 passing) happened this session** — previously unverified in this project's history.
- `patal-export`: tiled, true-scale PDF. Hand-rolled dependency-free writer (no `/CreationDate`, pinned `/ID`, so the golden file is a pure function of the geometry), `Mm`/`Pt` newtypes with one conversion, all-or-nothing emission behind a named `ExportError`, a calibration square on every sheet. Draws the kernel's `CutLine` — a newtype with no public constructor, so no second implementation of the cut line is representable. Verified against pdfium: the 50 mm square renders 50.004 mm, the 200 mm rule 200.008 mm. **Not yet validated on paper** — that is the next step, and it is the only one that counts.
- `apps/desktop`: draws a bodice front live with tolerance/seam-allowance sliders, reports per-frame cost vs. 120Hz budget, writes/reads real `.patal` files, exports the demo piece to a tiled PDF via temp-file-plus-rename.
- Repo housekeeping: Obsidian vault folded into `docs/` (versioned, no separate vault), duplicate Desktop checkout replaced with a directory junction. 33-commit history, pushed, CI green across all 5 jobs (engine ubuntu/Windows, desktop, native, advisories).

**Not started:** parametric constraint/propagation solver (the biggest unbuilt pillar — what makes this a real pattern CAD system vs. a drawing tool), DXF-AAMA/ASTM export, multi-piece nesting (tiled PDF gives each piece its own grid), grading, pattern primitives (darts/notches/grainlines/pleats/facings), Metal rendering, multi-device sync, the AI collaborator layer, any visual identity.

**Known gap:** no competitive analysis has ever been written down. Seamly2D/Valentina (free, ~13 years old, parametric) and Freesewing already ship DXF-AAMA export and tiled PDF — on the current axis (draw a polygon, offset a seam allowance) Pātāl is behind a free incumbent. This is the top open item.

## Active plan (as of today)

Ultraplan "The Wedge and Validation Wave" (`docs/plans/2026-08-13-wedge-and-validation-wave-ultraplan.md`), sequenced as: (1) decide what a `PatternPiece` stores — currently a flattened boundary, not the authored `SeamPath`, which blocks editing a saved file back into curves; (2) draft a real bodice block in Seamly2D and Freesewing, write down the actual competitive wedge as ADR-006, _then_ freeze the v2 document schema informed by that; (3) build export (DXF-AAMA/ASTM + tiled PDF with a calibration square) as the cheapest real validation path — print a piece at true scale, hand it to a pattern maker. Constraint: export lives in the Rust core (never the Tauri harness) and consumes the kernel's cut line rather than recomputing one.

## Note on this doc

This is a snapshot, not a live source of truth — `docs/status.md` in the repo is. Re-read the repo before relying on specifics here in a future session.