---
id: ADR-003
title: Curve Representation — two layers, not one
status: accepted
date: 2026-08-12
tags: [adr, geometry, cut-path]
---

# ADR-003 — Curve Representation

## Status
**Accepted** — 2026-08-12.

## Context

Necklines, armholes, sleeve caps and hems are curves. Until now they could only be
faked as polylines, with no way to edit them back into curves afterwards — a
neckline drawn as forty points is forty points forever.

`PatternBoundary` is the type whose output becomes a cutting line. It is 754 lines
with 25 tests, and it carries the invariants that keep a wrong number from reaching
cloth: construction-only validity, private points, and a refusal to return a
plausible-looking answer from a fallible operation.

The obvious move is to teach it about curves — make its points a sequence of
segments. An earlier draft of the hardening plan did exactly that, and it was
vetoed during the plan's own risk audit. Adding curves to the kernel puts
curve–curve intersection on the cut path and requires rewriting `self_intersects`,
`signed_area`, `winding` and `offset` **at once**, all four of which are
load-bearing and all four of which are tested against known-bad inputs that used to
produce silently wrong cutting lines.

## Decision

**Two layers. The kernel does not change.**

```text
  SeamPath  ──flatten(tolerance)──▶  PatternBoundary  ──offset──▶  cut line
  (authored)                          (manufactured)
```

- `SeamPath` is the authored representation: a start point and a list of
  `EdgeSegment`s, each a `Line` or a cubic `Bézier`. Private fields, construction-only
  validity, serde routed through `try_from` — the same doctrine `PatternBoundary`
  already follows.
- `flatten(tolerance_mm)` produces a `PatternBoundary` by adaptive subdivision.
- `flatten_for_offset(tolerance_mm, offset_mm)` tightens the tolerance so it still
  holds *after* the offset.
- `PatternBoundary` is untouched. All of its original tests pass unmodified, which is
  the evidence the cut path was not disturbed.

Cubic rather than quadratic: a cubic expresses an S-curve in one segment, which is
what a sleeve cap and a princess seam both are, and every drawing tool a designer has
used speaks cubics.

## Why this is not a compromise

This is the industry norm — parametric authoring, discretized manufacturing — and at
garment scale it is metrologically free. Industrial fabric cutting works to roughly
0.4mm. A flattening tolerance of 0.01mm is about forty times finer than any cutter can
execute and far finer than cloth can hold, so flatten-then-offset is
indistinguishable from true curve offsetting **for this application**.

That argument is scale-dependent and does not travel. It would not survive being moved
to optical tooling or PCB routing.

It also has a durable side benefit: `PatternBoundary` never changes shape, so the FFI
signatures that take one keep working permanently.

## Rejected — kurbo

kurbo is the obvious Rust curve library. Not adopted for the cut path. Its
`CubicOffset` was removed in favour of `offset_cubic`, and issue #344 — an endless
loop and NaN out of `fit_to_bezpath` — was open at the time of this decision, with
its status in 0.12 unconfirmed. A NaN on the cut path is precisely the failure mode
this crate's doctrine exists to prevent.

Recorded so the decision is not silently re-litigated later on stale information: if
kurbo is revisited, check #344 first rather than assuming it was about something else.

## The tolerance argument

Flattening error is a sagitta, proportional to the local radius. Offsetting changes
that radius: a region of curvature `κ` offset by a signed distance `d` along the
outward normal has its error scaled by `1 + d·κ`.

The sign of `d` decides *which* regions grow, not whether any do. A convex region
amplifies when offset outward and shrinks when inset; a concave region does the
opposite. So the worst case is `1 + |d|·|κ_max|` in **both** directions, and
tightening by its reciprocal is conservative either way.

`κ_max` is estimated by sampling, not solved. A cubic's curvature extremum is a root
of a high-degree polynomial, and cubics admit cusps where curvature is unbounded, so
there is no finite analytic bound to compute for arbitrary input.

**Treat the formula as an implementation detail and the oracle as the specification.**
`engine/crates/geometry/tests/curve_oracle.rs` checks a circle — the one shape whose
exact offset is known in closed form — across radii, tolerances, and all four
combinations of curvature sign and offset direction.

Two things about that oracle are worth carrying forward:

- **Four cubics is not enough to be an oracle.** A cubic cannot represent a circular
  arc exactly; a quarter-arc is off by 2.7e-4 of the radius, which on a 1000mm circle
  is 0.27mm against a 0.001mm tolerance. A test built that way measures how badly
  cubics approximate circles. The oracle uses 32 arcs (~1e-9 relative) and pins the
  quarter-arc figure in its own test so the error budget is explicit.
- **The convex/outset case alone proves nothing about the others.** An earlier version
  swept only positive `d` on circles, which are convex everywhere, so it never tested
  an inset and never tested a concave region — exactly the two cases where the
  argument above is not trivially true.

## Consequence this layer owns, and has not solved

**Flattening finely enough for accuracy can make a shape un-offsettable.**

A corner of turn angle `θ` consumes `d·tan(θ/2)` of length from each adjacent edge,
which at 90° is `d` exactly. Tightening the tolerance shortens chords; once a chord
next to a sharp corner is shorter than the seam allowance, that edge reverses and the
kernel correctly refuses with `OffsetCollapsed`.

Measured: a square with a semicircular bite, at a 0.5mm allowance, succeeds at a
0.01mm tolerance and fails at 0.001mm. Pinned as
`a_chord_shorter_than_the_allowance_at_a_sharp_corner_collapses`.

The kernel is not wrong — the polygon it was handed really does collapse. But it means
**tolerance cannot be chosen for accuracy alone**: it also sets a floor on the seam
allowance a piece can carry at a sharp corner. Nothing in the two-layer design
anticipated this, and nothing currently enforces it.

Options when this is picked up, none yet chosen: enforce a minimum chord length in
`flatten_for_offset` derived from the offset distance; detect sharp corners and
subdivide asymmetrically around them; or offset the authored curve directly at
corners and flatten only the smooth spans. The last is the most correct and the most
work.

## Consequences

- **Positive:** the kernel and its tests are untouched, so the cut path's proven
  behaviour is preserved. `PatternBoundary`'s wire format is permanently stable.
  Authored curves stay editable as curves.
- **Positive:** measured cost is not a concern at garment scale — about 1% of a 120Hz
  frame at manufacturing tolerance. See ADR-001 and `benches/drag_loop.rs`; the
  planned coarse-preview-during-drag optimisation was dropped on that evidence.
- **Negative:** two representations of the same edge exist, and something must decide
  when to re-flatten. Today nothing caches; that is fine at the measured cost and will
  need revisiting when there are many pieces on a canvas.
- **Negative:** the tolerance/allowance interaction above is a real, unsolved
  constraint on the layer.
- **Open:** node continuity between adjacent segments (smooth vs corner) is not
  modelled. A designer dragging a handle across a smooth join currently breaks
  tangency with nothing to prevent it. Needed before a real curve editor.
