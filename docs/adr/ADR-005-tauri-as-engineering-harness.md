---
id: ADR-005
title: Tauri as an engineering harness, not a shipping target
status: accepted
date: 2026-08-12
tags: [adr, tooling, process]
---

# ADR-005 — Tauri as an engineering harness

## Status
**Accepted** — 2026-08-12.

**This does not reverse ADR-001.** Read that sentence before the rest of this file.

## Context

ADR-001 rejected the Rust-all-the-way path (Dioxus / Tauri) as a *shipping* target,
on the grounds that its ceiling on native feel sits below Target 1's quality bar. That
judgement stands and is not reopened here.

What happened afterwards is that the rejection quietly widened. The Tauri app was
described as "frozen", the hardening plan treated it as a compile gate and nothing
more, and the *development* platform inherited a decision that had only ever been made
about the *shipping* platform.

The facts that decision ignored:

- **There is no macOS toolchain on the machine Pātāl is developed on.** `apps/native`
  has never been compiled locally and cannot be. CI is its only build, and until very
  recently CI had never run `swift build` at all.
- **The Tauri app links the engine crates directly**, as path dependencies, with no
  FFI boundary in between. No bindings to generate, no xcframework to build.
- **It runs on Windows today**, which is the only platform anyone can actually watch
  it on.
- Curves, seam allowances, and the document format had no visual consumer whatsoever.
  Every claim about them was a test assertion.

So the only thing that could show a human what the engine produces had been demoted to
a lint target, while the thing that was supposed to show them could not be built at
all.

## Decision

**Unfreeze `apps/desktop` as an explicitly non-shipping, disposable engineering
harness.**

It exists to answer questions that are expensive to answer any other way:

- Does that neckline actually look right at a 0.4mm tolerance?
- What does a seam allowance that exceeds the curvature look like when it fails?
- Does a `.patal` file survive a round trip through a real disk?

It draws a bodice front — the same four-cubic shape `benches/drag_loop.rs` measures —
with sliders for flattening tolerance and seam allowance, a live vertex count and
per-frame cost against the same 120Hz budget the benchmark reports, and a button that
writes a real `.patal` file and reads it back.

### Rules that keep it from becoming a product

1. **No product decisions here.** If a question about how Pātāl should behave gets
   answered in this app, the answer belongs in the engine or in an ADR, not in
   `src/main.ts`.
2. **Never reimplement engine logic.** The harness calls `patal-geometry`; it does not
   compute geometry. The Swift package once carried a second copy of the offset kernel
   and it was deleted for exactly this reason.
3. **Show the engine's errors verbatim.** Paraphrasing them starts a second error
   vocabulary that has to be kept in sync with the first.
4. **Delete it without ceremony** when the native app can answer the same questions.
   Nothing here is owed a migration.

## What this does not mean

- It does not make Tauri a shipping target. ADR-001's rejection is unchanged.
- It does not make `apps/desktop` a product surface, and no visual decision made in it
  carries weight.
- It does not relax ADR-001's per-frame FFI constraint. The harness has no FFI
  boundary at all, so it cannot test that constraint and its timings say nothing about
  it.

## Consequences

- **Positive:** geometry becomes visible on the platform it is developed on, at the
  cost of about ten lines of glue.
- **Positive:** the document format now meets an actual disk somewhere, which the
  engine deliberately does not do.
- **Positive:** it was already a CI compile gate; it now also runs tests there, so a
  change to a domain type's shape breaks loudly rather than rotting.
- **Negative:** a second UI exists, and someone will eventually mistake it for the
  product. The rules above and the README in that directory are the mitigation; if
  they stop being true, delete the app rather than promoting it.
- **Negative:** it adds a Node and Tauri toolchain to the set of things that must keep
  working. That cost was already being paid, since the app was already a compile gate.
