---
id: ADR-008
title: Export format decisions — tiled PDF first, and the conventions it commits to
status: accepted
date: 2026-08-14
tags: [adr, export, geometry, process]
---

# ADR-008 — Export format decisions

## Status
**Accepted** — 2026-08-14, covering the tiled PDF emitter shipped in `patal-export`.

**Partial by design.** The DXF-AAMA/ASTM half of the export question is *not* decided
here; see "Still open" at the end. This ADR records the decisions that are already
compiled into the repository, because they were being cited before they were written
down — `engine/crates/export/src/pdf.rs` referred to "ADR-008" as the record of the
stroke-centre rule while no such file existed. A rule you cannot read is not a rule.

## Context

Export is the first thing Pātāl produces whose correctness is not decided by its own
test suite. Everything upstream — the offset kernel, the seam-allowance validation,
the typed errors — is checkable by assertion. A printed pattern is checkable by a
steel rule, and the two can disagree.

That changes what a decision costs. A wrong number inside the engine is caught by a
test. A wrong number in the page transform produces a document that opens, renders,
prints, and is the wrong size, and the person who discovers it has already cut cloth.

Tiled PDF was taken before DXF because it is the cheapest route to that external check
(`status.md`, `roadmap.md`): pure Rust, no Mac, headlessly testable, and it cannot be
faked by a passing test suite.

## Decision

### The artifact is true scale or it is nothing

A millimetre in the model is a millimetre on the paper. **There is no scale parameter
and no fit-to-page**, and adding one is a change to this ADR, not an implementation
detail. The single job of a pattern exporter is that one guarantee, and a `scale`
argument is how it becomes conditional on a value nobody re-checks at the printer.

If a not-to-scale preview is ever genuinely needed, it must stamp `NOT TO SCALE` on
every page and in the filename, and it can be added then, loudly.

### The true line is the centre of the stroke

This is the convention `pdf.rs` cites and the one a person needs when they put a rule
on the paper. Lines are stroked at **0.25 pt (~0.09 mm)** on an identity CTM.

Two things follow, and both are load-bearing:

- **Width is set in points, never under a millimetre-scaled matrix.** A `1 w` under an
  mm-scaled CTM is a one-millimetre line. A stroke is centred on its path, so that is
  half a millimetre of ambiguity on each side about where the scissors go — comparable
  to the 0.4 mm the geometry kernel treats as cutter tolerance.
- **Measurements are read centre-to-centre.** Outer-edge-to-outer-edge adds one line
  width to every reading in the same direction. At the declared tolerance of ±0.5 mm
  over 200 mm that is a fifth of the budget spent silently. This was observed for real:
  a pdfium raster of the golden fixture measured 50.12 mm outer-edge and 50.004 mm
  centre-to-centre on the same 50 mm square.

### Units are types, not a convention

`Mm` and `Pt` are distinct newtypes and `Mm::to_pt` is the only conversion in the
crate. A raw `f64` cannot reach the page transform. `to_pt` is deliberately not
`const`, so it does not inline away into call sites and stop appearing in a search for
where units change.

### Every page carries its own proof

Each content sheet reserves a strip at the foot for a **50 mm calibration square**, and
page one carries ruled baselines on **both axes** — 200 mm down the page, and across
the page whatever fits, **printed with its true length rather than a round one**.

On A4 the across-page rule is 186 mm, and the arithmetic is worth spelling out because
it looks off by four: the rule starts 4 mm inside the margin so its corner tick has
room, giving `210 − (10 + 4) − 10 = 186`, not the 190 mm of bare drawable width. A
200 mm horizontal rule does not exist on A4 at any margin. Printing the clamped number
instead of the requested one is the point — a rule labelled 200 mm that is physically
186 mm would turn this page from an instrument into the very defect it exists to catch.

Both axes, because printers do not scale equally in both directions: the feed direction
is driven by rollers and the carriage direction by a belt. A single rule cannot tell a
driver problem from a printer problem. Long lines, because a 1% error moves a 50 mm
mark by half a millimetre — inside the noise of reading a rule — and a 200 mm mark by
two millimetres, which is not.

The strip is **reserved, not overlaid**. A box labelled "measure me" sitting on top of
a line labelled "cut me" is a hazard rather than a cosmetic problem, so the drawable
window starts above the strip and the collision is structurally impossible.

### Assembly: overlap the crosses, do not trim

Sheets within a piece overlap by 10 mm. Registration crosses are drawn on shared model
grid lines, so **the same cross carries the same label on both sheets that show it**,
and lining up `x1y0` with `x1y0` *is* the assembly.

The rejected alternative is trim-to-frame and butt the edges, which is the classic
double count: it grows the piece by `overlap × (sheets − 1)` — 30 mm on a four-sheet
piece — and the result looks like a slightly generous pattern rather than a mistake.

### The document is a pure function of the pieces

No `/Info` dictionary, no `/CreationDate`, a pinned `/ID`, no compression, one number
formatter at four decimal places. Two exports of the same pattern are the same bytes,
on any machine, this year and next.

### No partial documents

Every cut line is derived before a byte is written, and page geometry is validated when
the `PageLayout` is constructed rather than when it is used. A six-piece PDF produced
because the seventh piece failed looks exactly like a correct one. `patal-export`
returns bytes and touches no `std::fs`; the caller writes through a temporary file and
a rename, so an I/O failure cannot leave a truncated PDF that still opens and prints.

### Export draws the kernel's cut line and cannot compute its own

`PatternPiece::cut_boundary()` returns a `CutLine` — a newtype with a private field and
no public constructor, mintable only inside `patal-pattern`. This is constraint C11
moved out of a review checklist and into the compiler.

## Rejected

**A general-purpose PDF crate.** Not dependency politics. The golden test compares
bytes, and a byte comparison is only worth having if the bytes are a function of the
geometry alone. Every such crate stamps a creation date, or compresses through a zlib
whose exact output is not a stability guarantee, or both — and a golden that goes red
on an unrelated dependency bump is one that gets re-blessed unread at two in the
morning, which protects nothing. Roughly 2% of the PDF specification is needed here:
lines, a rectangular clip, dashes, and base-14 text.

**Compression.** It would save bytes on a document nobody stores at scale, and cost the
ability to read the file in a text editor when the golden fails.

**Clipping the polygon in Rust at the tile edge.** Geometry that decides where a line
stops is a second opinion about the cut line. The PDF clip operator does it instead.

**Font embedding.** Base-14 Helvetica with `WinAnsiEncoding` means a piece named
`Pātāl` prints as `P?t?l`. That is a real limitation, accepted knowingly: no label in
this document is load-bearing, nothing anyone cuts along depends on it, and a font
subset is an order of magnitude more code than everything else in the writer for a
purely typographic gain. Revisit if piece names with macrons or Devanagari become
common.

## Consequences

Good: the true-scale claim is checkable on the artifact by someone who does not trust
it, with a ruler, without running the software. The golden test is stable enough to be
believed. Nothing downstream can invent a second cut line.

Costs: the calibration strip spends 58 mm of every sheet. Piece names outside Latin-1
degrade visibly. The PDF writer is ours to maintain, including its cross-reference
table — though at ~450 lines covering a fixed subset, that is a smaller surface than
the reproducibility fight it replaces.

**What this does not mean.** It does not mean export is validated. Every measurement
recorded in this repository was made by software about itself, including the pdfium
figures above. The claim this crate exists to make is settled by a steel rule on paper
from two printers, and that has not happened. See `docs/setup/printing.md`.

It also does not decide anything about nesting. Each piece gets its own tile grid at
its own origin; packing several pieces onto shared sheets is two-dimensional bin
packing with grain constraints, a different problem from a page transform, and it is
deferred in `TODOS.md`.

## Still open

**DXF-AAMA/ASTM is not decided here.** The wave blueprint (§3.11) expects this ADR to
also record the position of the captured Seamly2D reference — oracle or merely sample —
and that capture (§3.3) has not happened. Until it does there is no DXF decision to
record, and inventing one from a feature table is exactly the failure ADR-006 is being
held back to avoid. `TODOS.md` and `status.md` both point at this file for that answer;
they should keep pointing here, and this section should be replaced rather than the
reference removed.
