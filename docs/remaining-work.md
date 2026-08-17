---
title: Remaining work
tags: [checklist, planning, living]
updated: 2026-08-17
review: every session
---

# Remaining work

> [!important] Next action
> **Chunk A — the v2 shape freeze.** Twenty minutes. It unblocks Task 8, grading, and
> pattern primitives simultaneously, and nothing else in Chunk B or D can start without it.

**Living page — not a snapshot.** Tick boxes as work lands, edit lines as they stop being
true, and bump `updated:` in the frontmatter. If this page and reality disagree, this page
is wrong and should be fixed on the spot. Undated filename on purpose.

**Progress: 0 / 31 complete.** *(A 0/6 · B 0/6 · C 0/6 · D 0/5 · E 0/5 · F 0/3)*

**Not a fourth tracker.** [status.md](status.md) owns *state*, [roadmap.md](roadmap.md)
owns *why*, [TODOS.md](../TODOS.md) owns *deferred items with context*. This page owns one
thing: what is left, in what order, and what blocks what.

**Legend** — `S` under an hour · `M` a session · `L` multiple sessions ·
⛔ needs an operator decision · 🍎 needs macOS

---

## A — The v2 shape freeze ⛔ · 0/6

The gate everything downstream waits on. Cheap, and the highest-leverage item on this page.

- [ ] Add a temporary `#[test] fn print_v2_shape()` — a two-piece project with one cubic edge, one `Smooth` join, a grain line and a non-default tolerance, printed pretty `S`
- [ ] Run it and **read the actual JSON**, not the prose summary
- [ ] Check it against the six points in [status.md](status.md) — outline never a polygon · edge nested `{geometry, join}` · `join` omittable = corner · piece carries `id`/`grain`/`seam_allowance_mm`/`material` · project carries `flatten_tolerance_mm` · P-03/P-05/P-13 deliberately absent
- [ ] Confirm the load-bearing reading: **freezing v2 does not decide whether a dart is an object** — a dart is an additive field either way
- [ ] ⛔ **Sign, or revise the shape**
- [ ] Delete `print_v2_shape` — a review instrument, not a regression

---

## B — Finish the storage wave · 0/6

Tasks 8–12 of 12. Spec: [the execution plan](plans/2026-08-16-seampath-storage-execution-plan.md).
Tasks 1–7 shipped 2026-08-17. **Every "expect N tests" in that plan is understated by 2** —
PR #5 moved the baseline. Live count: **168**.

- [ ] **Task 8 — schema v2 + migration** `L` ⟵ blocked by **A**. Biggest task left (~394 lines of spec): version-tolerant loader, frozen historical shape, `TryFrom` dispatch that *rejects* wrong-version fields, migration as a pure function, v1 + v2 fixtures. Own session.
- [ ] **Task 9 — the harness proves the curves came back** `M` · Windows. `SaveReport` reports segment and cubic counts. Also where `cut_preview` gets rerouted through `Project::cut_boundary`, deliberately skipped in Task 7.
- [ ] ⛔ **Gate — Swift: mirror or delete.** Blueprint §6 says mirror. Counter worth weighing: `apps/native` is 555 lines, no Xcode project, never built outside CI, holds no geometry, `Codable` is Swift-to-Swift only. A mirror nothing exercises drifts silently — the argument that deleted the Swift offset kernel.
- [ ] **Task 10 — Swift mirrors the v2 shape** `M` 🍎 ⟵ only if the gate says mirror. Writable on Windows, verifiable only via CI's `native` job.
- [ ] **Task 11 — benchmarks measure the no-cache decision** `S`. Decides whether `PatternPiece` ever gets a cached boundary. Do not add one without this.
- [ ] **Task 12 — ADR-007 + doc close-out** `M` · **non-negotiable.** D6 (export's project-aware signature) currently lives only in a plan file and the ADR index. Must carry D1–D4, the C9 argument, D6 **with both rejections**, and the signed freeze.

---

## C — The validations that can say "the software is wrong" · 0/6

No Mac. No code. Stalled for weeks while the engine moved a great deal. **This is the
highest-risk gap in the project** — not the unbuilt features.

- [ ] **Print the calibration page** `S` — two printers, steel rule, ±0.5mm over 200mm declared in advance, record printer + driver. Runbook: [setup/printing.md](setup/printing.md)
- [ ] **Print a bodice block** `S`
- [ ] **Find a pattern maker** `L` — long lead time, unstarted across five sessions. Gates the only outside verdict this project can get.
- [ ] **Hand them the block, take the verdict** `M`
- [ ] **K3 — draft a bodice block in Seamly2D *and* Freesewing** `M` — both run on Windows. GUI work, not code. Critical path to Decision 2 (is a dart an object?) and to ADR-006.
- [ ] **ADR-006 — the competitive wedge** `M` ⟵ blocked by K3. Must come from the friction of drafting, never a feature table. On the axis Pātāl currently competes on it is behind a free thirteen-year-old incumbent, and that has never been written down.

---

## D — The next pillars · 0/5

All pure Rust, all testable headlessly on Windows.

- [ ] **Grading / size runs** `L` ⟵ blocked by **A**. Strongest next-wave candidate. *"A pattern tool that cannot grade is a drawing tool."*
- [ ] **Multi-piece nesting / lay plan** `L` — **fully unblocked now**: `GrainLine` shipped, the page transform exists, `export_tiled_pdf` already takes a whole `&Project`. 2D bin packing with rotation constrained by grain.
- [ ] **Pattern primitives — darts, notches, pleats, facings** `L` ⟵ blocked by **A**. The `Edge` container exists so each lands as a field, not a schema v3. That claim gets tested here.
- [ ] **DXF-AAMA/ASTM export** `L` — needs a Seamly2D reference capture, then [ADR-008](adr/ADR-008-export-format-decisions.md)'s "Still open" oracle-vs-sample ruling.
- [ ] **Parametric constraint solver** `L` — the largest single item, and what separates this from a drawing program with a garment theme. Nothing propagates today.

---

## E — Parked behind Mac access 🍎 · 0/5

Gates the *product surface*, not the engineering. Nothing in A–D depends on any of it.

- [ ] **Metal canvas** — [ADR-001](adr/ADR-001-stack-selection.md). Not `wgpu`.
- [ ] **XCFramework packaging** — `xcodebuild -create-xcframework`. Note: uniffi *binding generation* runs fine on Windows; only packaging and compiling the Swift are Mac-gated.
- [ ] **Xcode project + shipping iPhone / iPad / Mac apps**
- [ ] **Multi-device sync** — not specified, not urgent, no second device to sync to.
- [ ] **Intelligence layer** — deliberately last; it needs something worth acting on first.

---

## F — Hygiene · 0/3

- [ ] **Windows CI job for the Tauri harness** `S` — `desktop` runs on `macos-latest` only, so the harness is never built on the one platform it is actually used on. A Windows-specific break would be caught only by running it locally.
- [ ] **Schedule the harness's disposal** `S` — [ADR-005](adr/ADR-005-tauri-as-engineering-harness.md) calls Tauri disposable; every wave adds a command to it and nothing schedules the disposal, so "disposable" drifts toward "permanent". Probable trigger: when Metal renders a piece on a Mac.
- [ ] **`_to_delete/`** — verified safe to remove (its bundle holds only `1f91066`, an ancestor of `main`). Kept deliberately 2026-08-17. `rm -rf` is blocked by a permission guard; run it yourself if you want it gone.

---

## The short version

**A** is twenty minutes and unblocks **B**, plus grading and primitives in **D**.
**B** is roughly three sessions.
**C** is the one that can prove the whole thing wrong, needs no Mac and no code, and has not moved.

---

## Keeping this page true

The failure mode for a page like this is going stale quietly and then being trusted anyway.
Three habits prevent it:

1. **Tick on landing, not on starting.** A half-done item stays unticked.
2. **When a line stops being true, edit the line** — do not add a correction underneath it. Corrections belong in the execution plan, which is a dated record; this page is a current view.
3. **Update the progress counts and `updated:` when you touch it.** A count that disagrees with the boxes is the first sign nobody is reading it.
