---
title: Pattern Primitives — the candidate list
date: 2026-08-14
status: draft — pre-contact baseline
tags: [analysis, domain, schema, wedge]
---

# Pattern Primitives — the candidate list

> **This is step K2** of [the wedge and validation wave](../plans/2026-08-13-wedge-and-validation-wave-ultraplan.md).
> It enumerates the constructs a garment pattern needs, from domain knowledge, **before
> Seamly2D or Freesewing has been opened**. Blueprint decision 11 put it in that order on
> purpose: a textbook supplies the vocabulary, so the incumbents are left to answer only
> the question a textbook cannot — *does the tool persist this in its file, or merely draw
> it?* Anything an incumbent persists is a candidate schema v2 field.
>
> Written the other way round, this list would not be a checklist. It would be a
> transcription of Seamly2D's feature set with the serial numbers filed off, and ADR-006
> would be the feature table that [the ADR index](../adr/README.md) explicitly warns
> against. The commit that adds this file is the evidence of the ordering; it must land
> before the first Seamly2D screenshot does.

## What this file is for

Three consumers, and it is worth being explicit about which is which, because the file
gets *edited* by two of them rather than merely read.

| Step | Uses this how |
|---|---|
| **K3 / K4** — drafting the block in each tool | As the observation instrument. Each row carries a question specific enough to answer with a file citation. |
| **K5 / K6** — §3.4 inventory → ADR-006 | Fills the evidence column in, then converts each prior verdict into a real one. §3.4's acceptance criterion is that *every* row ends marked fold-in or defer, and every defer carries a reason. |
| **F1** — the v2 shape freeze ⛔ | Reads the blast-radius column. It is the reason this file exists at all: the freeze is the wave's only one-way door, and it should be the best-informed step in it. |

## Two rules that make it worth something

**A prior is not evidence.** Every row below carries a *prior verdict* — a judgment about
Pātāl's schema formed from domain knowledge alone. That is legitimate: what belongs in
this project's document format is a design call, and it is ours to make. What is **not**
ours to guess is what the incumbents actually do. Expectations about their behaviour are
quarantined in [Pre-registered guesses](#pre-registered-guesses-about-the-incumbents) at
the end, marked as guesses, and scored afterwards. A row whose prior turns out right must
still cite the file or API that showed it. "As expected" is not a citation.

**Additive rows must not delay the freeze.** The wave has exactly one irreversible step,
F1, and the temptation at a freeze is to hold it open until everything is known. The
blast-radius column is the discipline against that:

- **`additive`** — a new optional field. Migration maps it to `None` or a default. Adding
  it at v3 costs the same as adding it at v2. **An additive row can never justify halting F1.**
- **`structural`** — changes the type or arity of something that already exists, or changes
  what "the outline" *is*. Cheap now, a breaking migration later. **Only structural rows
  can halt F1**, and only if K6's evidence promotes them.
- **`none`** — no document field at all. Renderer, exporter, or application concern.

K6 supplies the verdicts. This file supplies the rule they get judged against, pinned in
advance so the rule cannot be adjusted to fit whatever the evidence turns out to be.

## How to read a row

Each construct gets a stable ID (`P-01`…) so K6, ADR-006 and §3.6 can cite it without
quoting it. ⚠️ marks a construct whose absence or mis-storage reaches cloth — the
cut-path standard this repo applies to anything a scissors follows.

- **Is** — what the construct is, in the domain.
- **Missing** — what breaks without it. Physical consequence where there is one.
- **Pātāl** — the current state, cited to source. `v1` is what ships today; `v2 planned`
  is what [the SeamPath blueprint](../plans/2026-08-13-seampath-storage-ultraplan.md)
  already commits to.
- **Ask** — the persistence question, phrased so a yes/no plus a citation closes it.
- **Prior** — verdict, blast radius, reason.

Verdicts: **IN-V2** (already committed by the SeamPath blueprint; not re-decided at K6) ·
**FOLD IN** · **DEFER** · **OUT OF SCHEMA** · **OPEN** (no honest prior without evidence —
the highest-value rows for K3/K4, because tool contact actually decides something).

## What Pātāl models today

The baseline every row is measured against. Read from source at `d6fd2ff`, not from memory.

**v1, shipping** — `engine/crates/pattern/src/lib.rs`:

```rust
Document  { schema_version: u32, project: Project }
Project   { name, pieces: Vec<PatternPiece>, measurements: Vec<Measurement>, materials: MaterialLibrary }
PatternPiece { name: String, boundary: PatternBoundary, seam_allowance_mm: f64, material: Option<MaterialId> }
Measurement  { name: String, value_mm: f64 }
```

`PatternBoundary` is a flat closed polygon. `seam_allowance_mm` is **one scalar for the
whole piece**, defaulting to 10mm (`lib.rs:210`). `cut_boundary()` is
`boundary.offset(seam_allowance_mm)`, returning a `CutLine` that is derived and never
stored. There is no identity on a piece, no relationship between two pieces, and no mark
of any kind anywhere in the document.

**v2, already committed** by the SeamPath blueprint and not re-opened here: the authored
`SeamPath` replaces the flattened boundary (§3.1, §3.6), `Join { Corner, Smooth }` as a
parallel array (§3.2), `GrainLine { angle_deg, anchor }` (§3.3), `PieceId` (§3.4),
`Project.flatten_tolerance_mm` (§3.5), and the v1→v2 migration (§3.7).

**The honest summary.** A `.patal` file describes an outline, one allowance, and a fabric.
Every construct below that a pattern maker would call ordinary is absent from it.

---

## A — The line the scissors follow

### P-01 · Authored outline ⚠️
- **Is.** The piece's edge as the designer drew it: straight runs and curves, not a
  polygon approximating them. A sleeve cap and a princess seam are both S-curves.
- **Missing.** A saved file cannot be edited back into curves; every reload degrades the
  drawing. This is the gap the SeamPath wave exists to close.
- **Pātāl.** v1 stores `PatternBoundary`, flat. v2 planned: `SeamPath` of
  `EdgeSegment::{Line, Cubic}` (§3.1, §3.6).
- **Ask.** Nothing — settled before this list was written.
- **Prior.** **IN-V2** · structural · already decided.

### P-02 · Seam line versus cut line — which one is authoritative ⚠️
- **Is.** Two different lines. The *seam line* (net, stitching line) is where the needle
  goes and is what the geometry of the garment is actually made of; the *cut line* is the
  seam line plus allowance and is what the scissors follow. A pattern system must pick one
  as authored and derive the other.
- **Missing.** Nothing visibly breaks — until allowance varies per edge (P-03), at which
  point deriving in the wrong direction means the *seam* moves when a *hem* is deepened.
  Home-sewing patterns are often distributed cut-line-authoritative with allowance already
  baked in and no net line recoverable; industrial patterns are net-authoritative.
- **Pātāl.** Net-authoritative, and correctly so: `boundary` is the seam line,
  `cut_boundary()` derives the cut line by offsetting outward, and `CutLine` is
  deliberately not `Serialize` — "a cut line is derived, never stored"
  (`pattern/src/lib.rs:105`).
  This is already the better half of the choice; it is simply never written down.
- **Ask.** Which line does the incumbent's file store? If both, which one does an edit
  move, and what happens to the other?
- **Prior.** **FOLD IN** as a *documented invariant*, not a field · none · The rule already
  holds in code and has no home in prose. It belongs in ADR-007 alongside what a piece
  stores. Storing a flag would be worse than useless: a file that could claim
  cut-authoritative while the type system derives outward would be a file that lies about
  where the scissors go.

### P-03 · Per-edge seam allowance ⚠️ — **the headline row**
- **Is.** Allowance is a property of a *seam*, not of a *piece*. A neckline is finished at
  6mm, a side seam at 10–15mm, a hem turned at 20–40mm. One garment routinely uses four
  different values on one piece.
- **Missing.** Either the hem is cut 30mm too short or every other seam is cut 30mm too
  wide. There is no single scalar that is right for a real piece, so the piece as stored is
  wrong somewhere by construction — quietly, and in a way that only shows up in cloth.
- **Pātāl.** v1 has one `f64` per piece, default 10mm, and nothing in v2 changes that.
- **Ask.** Does the file record allowance per edge, or one value per piece, or none at all
  (with the drawn outline already being the cut line)? Is the value attached to the edge,
  or to a separately-stored allowance path?
- **Prior.** **FOLD IN** · **structural** · One of only three decisions that can
  legitimately halt F1 — see *What F1 actually has to decide*, below. The reason is not
  that it is important; several rows are important. It is that its storage shape is not
  additive.

**Why P-03 is structural, and the finding that follows from it.**

An `Option<f64>` bolted onto the piece would be additive. Per-*edge* allowance is not: it
is an attribute of each element of `SeamPath.segments`, and v2 as currently planned already
introduces one such attribute — `joins: Vec<Join>`, a parallel array with the invariant
`len == segments.len()` (§3.2). Fold P-03 in later and there is a second parallel array.
Notches (P-13) attach to edges too, and that would be a third.

Three arrays that must stay the same length is three chances to get it wrong, and every
edit that splits or merges a segment has to maintain all of them. The alternative costs
nothing extra today:

```rust
// v2 as currently planned — each attribute is its own array, each with its own invariant
struct SeamPath { start: Point2, segments: Vec<EdgeSegment>, joins: Vec<Join> }
//                                                           ^ then allowances, then
//                                                             notch anchors, then folds…

// the alternative — the edge carries its own attributes, and grows without a new invariant
struct SeamPath { start: Point2, edges: Vec<Edge> }
struct Edge { geometry: EdgeSegment, join: Join }
```

**So the recommendation to F1 is: decide the edge-attribute container shape at v2, even if
`Join` is the only attribute that ships in it.** Choosing `Vec<Edge>` and populating it
with joins alone is a v2 decision with no extra work. Choosing parallel arrays and
discovering P-03 at v3 is a breaking migration of the one type the whole document is built
out of. This is derivable today, from the code and the domain, with no Mac, no printer and
no Seamly2D — which is what makes it K2's actual deliverable rather than a note.

**One correction to the cost estimate, offered with its provenance.** The obvious objection
is that variable-width offset is a hard geometry problem and the kernel is untouchable this
wave. Reading `PatternBoundary::offset_with_miter_limit` (`geometry/src/lib.rs:363-473`),
that appears not to be the case here: the loop already computes each edge's own outward
normal and pushes it by a scalar that merely *happens* to be constant, then re-intersects
consecutive offset edges. Per-edge distances would make `push` an array and leave the
re-intersection and the collapse check untouched. What genuinely needs thought is the
corner: `miter_cap` is `miter_limit * distance_mm.abs()` and `JOIN_MERGE_FRACTION *
distance_mm.abs()` — both take a single distance, and at a corner between a 6mm edge and a
40mm edge there are two. A rule has to be chosen and defended.

**This assessment is from reading, not from running.** It is offered to inform the freeze,
not to authorise the change. Nothing here licenses touching the kernel during this wave —
and it does not need to: **storage and computation can land apart.** v2 can store per-edge
allowance while `cut_boundary()` continues to offset uniformly, refusing loudly when the
stored values disagree. Storing it now costs a field. Not storing it costs a v3.

### P-04 · Allowance corner treatment ⚠️
- **Is.** Where two allowances of different width meet, the corner has to be resolved —
  mitred, extended square, or butted. At a hem-to-side-seam corner the difference between
  treatments is centimetres of cloth.
- **Missing.** The corner comes out at whatever the offset algorithm happened to do, which
  is a rendering artefact rather than a decision.
- **Pātāl.** The kernel already has a position: mitre when affordable, bevel when the mitre
  runs away, "never fabricate a vertex that lies on neither line"
  (`geometry/src/lib.rs:423`). It is
  a good default and it is not expressible per corner.
- **Ask.** Is corner treatment recorded per corner, or is it the offset engine's business?
- **Prior.** **DEFER** · additive · A per-corner override is a real feature and a plausible
  v3 `Option<CornerTreatment>`. It cannot be reasoned about honestly until P-03 exists,
  because with one uniform allowance the interesting corners do not arise. Deferring it is
  cheap precisely because P-03 is being taken seriously now.

### P-05 · Fold edge / cut on fold ⚠️
- **Is.** An edge laid on the fabric fold. The stored piece is half the real piece; the
  other half is a mirror. Two consequences, both physical: the edge takes **no allowance**,
  and the finished piece is twice as wide as the outline says.
- **Missing.** The two failures are opposite and both severe. Offset a fold edge and the
  centre front gains a seam allowance that becomes a bulge or a false seam. Ignore the
  mirroring and the yardage, the lay plan and the grain check are all computed against half
  a garment.
- **Pātāl.** Nothing. Every edge gets the same allowance, and the concept of a piece that
  is half of itself does not exist.
- **Ask.** Is "on fold" a property of an edge in the file, a separate axis object, or a
  drawn annotation with no machine meaning?
- **Prior.** **FOLD IN** · **structural**, and for the same reason as P-03: it is per-edge.
  It is the strongest argument that P-03's container question is not a one-off — the edge
  needs *at least* a join, an allowance and a fold flag, which is three attributes before
  anyone gets ambitious. Note that "fold edge" is not the same as "allowance 0mm": a
  zero-allowance edge is cut where it is drawn, a fold edge is not cut at all.

### P-06 · Internal construction lines
- **Is.** Geometry drawn on a piece that is not cut: dart legs, pleat folds, pocket and
  buttonhole placement, a lapel roll line, style lines, the centre-front line.
- **Missing.** Everything positional in the garment becomes guesswork at the sewing table.
- **Pātāl.** Nothing. A piece is exactly one closed outline.
- **Ask.** Are these stored as first-class objects with a type, as anonymous geometry on a
  layer, or drawn and thrown away?
- **Prior.** **FOLD IN** · additive · A `Vec<InternalLine>` on the piece, `None`/empty in
  the migration. Being additive it must not hold F1 (see the rule above) — but it is cheap
  enough that there is no reason to defer it either, and P-09 and P-10 both need somewhere
  to put their legs and folds.

### P-07 · Interior cut-outs
- **Is.** A closed hole strictly inside the piece — a keyhole neckline opening, a slot.
- **Missing.** Rare in garments; the usual answer is a facing rather than a hole.
- **Pātāl.** Impossible to express: a piece is one closed path.
- **Ask.** Does a piece hold one path or several, and if several, is there a rule about
  which is the outer one?
- **Prior.** **DEFER** · structural · Real but rare, and the migration from one path to
  many is the same shape of work whenever it is done. Deferring costs a later breaking
  change on a construct most garments never use; folding it in now costs winding rules,
  containment validation and an offset that must push holes inward. Not worth it against
  a bodice block. Revisit if K5's DXF layer inventory shows the format assumes it.

### P-08 · Edge identity and seam pairing
- **Is.** The fact that *this* edge of the front sews to *that* edge of the back. It is
  what makes a set of pieces a garment rather than a pile of shapes, and it is what lets a
  tool answer "do these two seams match in length?" — the `walk` a pattern maker does by
  hand with a tape.
- **Missing.** Nothing can be verified across pieces. Every consistency check a pattern
  maker performs stays manual, and an edit to one piece cannot propagate to its partner.
- **Pātāl.** `Project.pieces` is a `Vec` with no relation of any kind between elements.
  This is worth stating plainly, because it is the largest gap in the repository relative
  to the project's own claims: the memorandum's "living system composed of interconnected
  relationships" and [docs/README](../README.md)'s "a pattern is *a system of
  relationships*" both describe something the document format currently has no way to
  express at all.
- **Ask.** Does the incumbent's file record which edges sew together — as a first-class
  relation, or only implicitly via notches (P-13) and shared construction points?
- **Prior.** **OPEN** · structural · This is the most consequential OPEN row in the list
  and it is deliberately unresolved. It is where ADR-006's wedge most plausibly lives: if
  neither incumbent models seam pairing as data, that is a real axis on which Pātāl could
  be different rather than merely later. If both do, the memorandum's central claim is
  table stakes and ADR-006 has to say so. Either finding is worth the wave. **This is the
  row K3 and K4 should be watching hardest.**

---

## B — Making cloth three-dimensional

Cloth is flat and a body is not. Everything in this family is a way of consuming flatness:
a dart removes it, a pleat folds it, gathering distributes it. These are the constructs
that separate a pattern from a shape.

### P-09 · Dart ⚠️
- **Is.** A wedge taken out of the cloth so the flat piece can curve over a body. It has an
  apex, two legs, and an *intake* (the width removed at the mouth). Critically, a dart is
  not two lines drawn on a piece: when it is folded shut the boundary it crosses moves, so
  the seam line at the dart mouth must be **trued** — the pattern is folded closed and the
  edge cut through, which leaves a small notch or peak in the outline rather than a
  straight run.
- **Missing.** The piece is cut with an untrued edge. When the dart is sewn, the seam line
  is short and the hem or waist steps out of alignment with its neighbour. This is the
  classic and unmistakable sign of a pattern drawn rather than engineered.
- **Pātāl.** Nothing at all, in either version. `SCHEMA_VERSION`'s own doc comment lists
  darts among the reasons "version 2 is close to certain" (`pattern/src/lib.rs:27`).
  Worth noting where the word *does* appear: throughout the geometry kernel, always as the
  shape that stresses the offset algorithm — a 6° dart apex is the acute corner that makes
  a mitre run away (`geometry/src/lib.rs:224`, `:681`, and a property test at
  `geometry/tests/properties.rs:295`). The kernel knows exactly what a dart does to an
  offset. The document format has no idea what a dart is.
- **Ask.** Is a dart an object with an apex and an intake, or is it two internal lines plus
  a manually-trued outline? Does the file record the dart such that closing it recomputes
  the boundary, or is truing the designer's problem?
- **Prior.** **OPEN**, leaning fold-in · **structural** if darts are objects, additive if
  they are internal lines · The two answers are genuinely different schemas. If a dart is an
  object, the authored outline is no longer authoritative — the cut line becomes a function
  of outline *and* darts, which touches P-02, P-03 and `cut_boundary()` at once. If it is
  internal lines plus a trued outline, it is P-06 with a label. **This is the second row
  that could halt F1**, and the evidence needed to decide it is exactly what K3 produces:
  draft a bodice block, which cannot be done without a bust dart.

### P-10 · Pleat and tuck ⚠️
- **Is.** Fullness folded rather than removed. A pleat has a depth, a direction (knife, box,
  inverted) and a fold line plus a placement line. Like a dart, the edge it crosses must be
  trued across the folded state.
- **Missing.** Same failure as P-09: the crossing edge is wrong, and the piece is short of
  cloth by the pleat depth wherever the pleat was not allowed for.
- **Pātāl.** Nothing.
- **Ask.** Is pleat depth and direction stored, or is the extra width already drawn into the
  outline with only the fold lines annotated?
- **Prior.** **DEFER** · additive · Structurally it is P-09's problem again but a bodice
  block has no pleats, so this wave will produce no evidence about it. Deferring a construct
  the wave cannot observe is honest; folding one in on speculation is how a schema acquires
  fields nobody validates. Reconsider when a skirt or trouser block is drafted.

### P-11 · Ease and gathering along an edge
- **Is.** Two edges that sew together at *different* lengths, on purpose. A sleeve cap is
  cut longer than the armscye it sets into — the surplus is eased in, not pleated — and a
  gathered skirt may be two or three times its waistband. Ease is a **relationship between
  two edges**, not a mark on one.
- **Missing.** Without it, the only way to express "these two seams do not match and that is
  correct" is a note. Every automatic length check either fires falsely or is not written.
- **Pātāl.** Nothing, and nothing could hold it: P-08 shows there is no relation between
  pieces for an ease value to live on.
- **Ask.** Is ease a stored quantity, a derived difference the tool reports, or purely the
  designer's knowledge?
- **Prior.** **DEFER** · additive, and conditional on P-08 · A field with nowhere to attach
  is not a field. If K6 promotes P-08, this becomes an attribute of the pairing and should
  be revisited in the same breath; if it does not, ease has no home and deferring is the
  only coherent answer. Recorded here so that F1 sees the dependency rather than the field.

### P-12 · Shape operations — dart pivot, slash and spread
- **Is.** Not stored constructs but the core *verbs* of pattern making. Dart manipulation
  rotates intake from one location to another around the apex — a waist dart becomes a
  side-seam dart, the shaping is identical, the garment is different. Slash-and-spread cuts
  the pattern and opens it to add fullness.
- **Missing.** The tool draws patterns instead of making them. A designer can still achieve
  the result by hand, at the cost of doing it by hand.
- **Pātāl.** Nothing, and it is worth being clear that this is an *engine* gap rather than a
  format gap — until the question below is answered.
- **Ask.** **The persistence question is the whole question here.** Does the file record
  that a dart was pivoted — an operation, replayable and adjustable — or only the resulting
  geometry? A tool that stores operations has a history model; a tool that stores results
  has a drawing. This is a much deeper difference between the incumbents than any feature.
- **Prior.** **OPEN** · none, or structural if operations are stored · Almost certainly
  outside the schema for v2 — an operation log is a different kind of document — but the
  answer bears directly on ADR-006, because "every creative decision remains editable,
  interconnected, and reversible" (memorandum) is a claim about exactly this. If both
  incumbents store operations, Pātāl storing results is a real deficit and the wedge is
  elsewhere. Pairs with P-26 and P-32.

---

## C — The marks a cutting room reads

This family is where the wave's two tracks meet. §3.10 predicts the sewer handed a printed
block will ask for exactly these — the blueprint's own decision 7 accepted piece metadata on
the page for that reason — and DXF-AAMA defines several of them as specific entities, which
is what makes the K5 layer inventory worth capturing.

### P-13 · Notch ⚠️
- **Is.** A registration mark on the boundary, cut or drilled, that tells the sewer which
  point on this edge meets which point on the next piece's. Conventions carry meaning: one
  notch versus two distinguishes front from back on a sleeve, and notch *type* (slit, V,
  T) differs by cutting method.
- **Missing.** Pieces are joined by eye. On a curved seam — a sleeve into an armscye,
  where one edge is eased into the other — joining by eye is not a degraded workflow, it is
  a failed one.
- **Pātāl.** Nothing.
- **Ask.** Is a notch stored, and **how is it positioned**? Distance along the path from a
  known start, a parameter on a specific edge, or an absolute point that happens to sit on
  the boundary?
- **Prior.** **FOLD IN** · **structural**, and the reason is that positioning question ·
  An absolute `Point2` is additive and wrong: move the outline and the notch silently stays
  behind, off the edge, which is the plausible-looking-wrong-value C1 forbids. A position
  parameterised on the edge (`edge index`, `t`) is correct and couples notches to
  `SeamPath.segments` — the third attribute wanting a home in P-03's container. Whichever
  is chosen, deriving it wrong is a cut-path defect: the notch is a mark someone cuts.

### P-14 · Drill hole / punch mark ⚠️
- **Is.** An interior registration mark, for things that are not on an edge: a dart apex, a
  pocket corner. Industrially a drill passes through the whole lay.
- **Missing.** Interior positions (P-06) can be drawn but not transferred to cloth.
- **Pātāl.** Nothing.
- **Ask.** Stored as a distinct entity, or as a zero-length internal line?
- **Prior.** **FOLD IN** · additive · A `Vec<Point2>` of interior marks, or a variant of
  P-06's internal-geometry collection. It carries one domain fact worth recording in the
  type's documentation rather than discovering later: **a drill hole is deliberately set
  back from the point it marks** — roughly 10–15mm short of a dart apex — because the hole
  is permanent and must not show on the finished garment. Storing the apex and drilling the
  apex are different things, and a tool that conflates them punches a visible hole in the
  front of every bodice it makes.

### P-15 · Grain line
- **Is.** The direction of the fabric's warp on the piece. Cloth is stiff along the warp,
  gives across the weft, and gives most on the bias; a piece laid at the wrong angle hangs
  wrong and cannot be fixed afterwards.
- **Missing.** The cutter guesses, and the garment twists.
- **Pātāl.** v2 planned: `GrainLine { angle_deg, anchor }` on `PatternPiece.grain`,
  normalised into `[0, 360)` because a grain line is directional rather than axial (§3.3).
- **Ask.** Nothing — settled.
- **Prior.** **IN-V2** · additive · already decided.

### P-16 · Nap and directional layout
- **Is.** Some cloth has a direction: velvet, corduroy, one-way prints. Every piece must
  then be laid the same way up, which changes the lay plan and the yardage.
- **Missing.** A velvet jacket with one panel laid upside down, which reads as two different
  colours under light and is unrecoverable.
- **Pātāl.** Nothing — but this is the row that belongs to the **material**, not the piece,
  and `Material` already exists with `drape`, `rigidity` and `surface_texture`
  (`materials/src/lib.rs`). It is the natural home.
- **Ask.** Does the tool model nap at all, and if so on the fabric or on the piece?
- **Prior.** **FOLD IN** · additive · A `directional: bool` (or a small enum covering
  one-way and two-way-nap) on `Material`. It is cheap, it has an obvious home in a type
  that already exists, and §3.3's normalisation decision was made *for* it — the blueprint
  argued grain must keep 190° distinct from 10° precisely because napped fabrics require it.
  Storing the constraint that justifies the decision, next to the decision, costs one field.

### P-17 · Cut quantity and handedness ⚠️
- **Is.** How many of this piece to cut, and whether they are identical or mirrored. "Cut
  2" of an asymmetric front means one left and one right — a *pair*, cut face-to-face. "Cut
  2" of a symmetric piece means two of the same.
- **Missing.** Two left fronts. The garment cannot be assembled and the cloth is spent.
  This is among the most common real-world cutting errors and it is purely an information
  failure.
- **Pātāl.** Nothing. A piece exists once and says nothing about how many times it is cut
  or in which reflection.
- **Ask.** Is cut count on the piece? Is mirroring distinguished from repetition, or is
  "cut 2" a string on a label?
- **Prior.** **FOLD IN** · additive · Small, unambiguous, physical, and with a real failure
  mode behind it. It also raises the question P-21 answers: once a piece can be cut more
  than once, "cut 2 self and 2 interfacing" is expressible only if quantity and material
  travel together.

---

## D — Identity and instruction

### P-18 · Piece identity
- **Is.** A stable id, independent of the name, so a piece can be referenced without being
  renamed out from under the reference.
- **Pātāl.** v2 planned: `PieceId`, UUID-backed, copying `MaterialId` exactly (§3.4).
  [ADR-004](../adr/ADR-004-document-format.md) records the divergence this closes — Swift's
  piece has a `UUID`, Rust's has no identity field at all.
- **Prior.** **IN-V2** · additive · already decided.

### P-19 · Piece name
- **Pātāl.** v1 has `name: String`. Present, used in errors, used on the printed page.
- **Prior.** No action. Listed for completeness so the summary table is a full census
  rather than a list of gaps.

### P-20 · Piece role — self, lining, interfacing, facing, underlining
- **Is.** What layer of the garment this piece belongs to. A facing is a separate piece;
  interfacing is usually the same shape as the piece it stiffens, cut from a different
  material and often trimmed back by the seam allowance.
- **Missing.** The pattern cannot express a lined or interfaced garment, which is to say
  most tailored garments.
- **Pātāl.** Nothing. There is no notion of a piece being *about* another piece.
- **Ask.** Is role stored as an attribute, or is it convention in the piece's name
  ("Front Facing")? Is an interfacing piece derived from its parent, or drawn separately
  and maintained by hand?
- **Prior.** **DEFER** · additive · The attribute is trivially additive and could land any
  time. The *derivation* — interfacing as a function of another piece — is a relationship,
  and it runs into P-08's absence exactly as P-11 does. Deferring the label alone is the
  honest call: adding a role enum that nothing reads is a field that only costs bytes,
  which is the failure §3.4 of the SeamPath blueprint argued against for `PieceId`.

### P-21 · Material per cut instance
- **Is.** One piece is often cut from more than one material — "cut 2 in self, 2 in lining,
  1 in interfacing". Material belongs to the *cut*, not to the piece.
- **Missing.** The lined version of any piece must be duplicated as a second piece whose
  geometry then has to be kept in sync by hand.
- **Pātāl.** `PatternPiece.material: Option<MaterialId>` — exactly one, or none. ADR-004
  made this a reference rather than an embedded copy, which was the right fix to a
  different problem; the arity was never examined.
- **Ask.** Does the file bind material to the piece or to a cut instruction? If the latter,
  what is that entity called and what else does it carry?
- **Prior.** **OPEN**, leaning fold-in with P-17 · **structural** · P-17 and P-21 are the
  same row seen twice: the moment a piece is cut more than once, quantity, material,
  handedness and role (P-20) all attach to the *cut*, not to the piece — which means a new
  entity, `Vec<CutInstruction>`, and demoting `PatternPiece.material` from a field to a
  default. That is a breaking change to an existing field, so it is structural, so **it is
  the third row that could halt F1**. It is marked OPEN rather than fold-in because a
  bodice block is unlined and this wave may produce no evidence either way; K6 should say so
  explicitly rather than resolve it on taste.

### P-22 · Annotation
- **Is.** Free text placed on the piece: "clip to dot", "ease between notches", a style
  number.
- **Pātāl.** Nothing.
- **Ask.** Is text an object with a position, or baked into the exported drawing?
- **Prior.** **FOLD IN** · additive · Cheap, obviously useful, and unblocked. It is also
  the escape hatch for every deferred row in this list: a construct Pātāl cannot model can
  at least be written on the piece by the designer, which is strictly better than losing it.

### P-23 · Printed title block
- **Is.** The stamp on each printed piece — name, size, grain direction, cut count, date.
  In production this is what identifies a piece of paper found on a table.
- **Pātāl.** `patal-export` prints the piece name and the calibration square
  ([ADR-008](../adr/ADR-008-export-format-decisions.md)); the rest has nothing to print.
- **Ask.** Not a persistence question. What does the incumbent's *printed output* carry?
- **Prior.** **OUT OF SCHEMA** · none · Every field in a title block is drawn from data
  owned by other rows — P-17, P-15, P-27, P-19. It is an export layout concern and it
  belongs to ADR-008, not to the document format. Recorded because V6's reviewer is likely
  to name it, and when they do, the answer should be "that is the exporter" rather than a
  new field.

---

## E — Sizing and parameters

This family is where the incumbents differ most from each other and from Pātāl, and where
ADR-006 will spend most of its argument. Both Seamly2D and Freesewing are *parametric*
systems: a pattern is a construction defined against measurements, not a set of coordinates.
Pātāl today stores coordinates.

### P-24 · Measurement set as a document
- **Is.** A named, reusable set of body measurements — a person, or a size standard —
  separate from the pattern that consumes it. Applying a different set to the same pattern
  is what makes a pattern *for* a body rather than *of* one.
- **Missing.** Measurements are trapped in one file and cannot be shared, versioned, or
  swapped. Every pattern is bespoke to whoever it was drafted on.
- **Pātāl.** `Project.measurements: Vec<Measurement>` — a flat list of `{name, value_mm}`
  inside the project. No vocabulary, no standard names, no separate document, no
  multi-size table.
- **Ask.** Is the measurement set a separate file? Is there a controlled vocabulary of
  measurement names, and does the pattern reference measurements by that name?
- **Prior.** **OPEN**, leaning fold-in as a *reference* · structural if the set becomes an
  external document · Splitting measurements out is a document-model change, not a field.
  The controlled-vocabulary half is the part with real domain content: "bust" and
  "chest circumference" being the same measurement under two names is a data-quality
  problem that only a shared vocabulary solves, and every parametric system has to solve it.

### P-25 · Parametric expression — **the deepest gap**
- **Is.** A construction point defined as a *formula* rather than a coordinate:
  `neck_width = bust/8 + 20mm`. The pattern is a program over measurements; changing the
  measurement re-runs it.
- **Missing.** Every pattern is drafted at one size for one body and cannot follow a
  measurement change. Grading (P-27) is then also impossible, because grading is the same
  mechanism applied across a size run.
- **Pātāl.** Nothing. Every point in a `SeamPath` is a literal `Point2`. This is the
  distance between the memorandum's "the engine converts creative intent into
  mathematically accurate construction geometry" and what the engine does, and it should
  be stated in ADR-006 in exactly those terms.
- **Ask.** How is a point defined in the file, and what is the expression language? Are
  expressions stored, or evaluated and discarded?
- **Prior.** **DEFER** · structural, and very large · This is not a v2 field; it is a
  different kind of document, and it is the largest unbuilt thing in the project after the
  constraint solver it implies. Deferring it is correct and must be said out loud rather
  than by omission — **the reason it is deferred is size, not unimportance.** The v2 freeze
  should nonetheless be taken with it in view: a format that stores literal points is not
  wrong, but it is a format a parametric v3 will have to migrate, and knowing that now is
  worth more than pretending otherwise.

### P-26 · Constraint and dependency graph
- **Is.** The structure that makes an edit propagate. Two different mechanisms are
  routinely confused: a **dependency DAG** (each point is a one-way function of earlier
  points and measurements — recompute in order) and a **constraint system** (relations are
  bidirectional and a solver finds a configuration satisfying them, as in mechanical CAD).
  They have different authoring feels, different failure modes, and different costs.
- **Missing.** "Changes made anywhere within the design should intelligently propagate
  throughout the project" (memorandum) is unimplemented and, without P-25, unimplementable.
- **Pātāl.** Nothing. The crate header says so plainly: "the constraint/propagation solver
  itself is a deliberately separate, larger milestone and is not yet implemented here"
  (`pattern/src/lib.rs:1-9`).
- **Ask.** Which of the two mechanisms does each incumbent use, and what does it do when a
  relation cannot be satisfied? §3.1 of the wave already pre-registers that last question
  as a friction-log item; it is the same question and the answers should be cross-checked.
- **Prior.** **DEFER** · none for v2 · Named here because **which of the two Pātāl builds
  is an open wedge question, not a settled one**, and ADR-006 is the right place to take a
  position. If both incumbents are DAG-based, a genuine constraint solver is a differentiator
  that is expensive but real; if one already solves, the wedge is elsewhere and the honest
  ADR says so.

### P-27 · Grade rules and size run
- **Is.** How the pattern changes across sizes: an increment per size at each grade point,
  applied along X and Y.
- **Missing.** One garment, one size. `docs/roadmap.md` puts it bluntly — "a pattern tool
  that cannot grade is a drawing tool".
- **Pātāl.** Nothing. Already tracked as a P3 item in [TODOS](../../TODOS.md), depending on
  the v2 schema being frozen.
- **Ask.** Are grade rules stored per point, per piece, or derived from re-evaluating P-25's
  expressions against a different measurement set? The second and third are different
  architectures and the choice is consequential.
- **Prior.** **DEFER** · structural when it lands · Correctly outside this wave. The one
  thing K5/K6 should extract is *which architecture* the incumbents use, because that
  choice constrains P-24 and P-25 and it is cheap to learn while the tools are open.

### P-28 · Ease — design and wearing
- **Is.** The difference between the body and the garment. *Wearing ease* is what makes
  movement possible; *design ease* is silhouette. Both are applied to measurements before
  the draft, and they are the reason a 90cm bust does not produce a 90cm garment.
- **Missing.** Ease is baked into whatever numbers the designer typed and cannot be
  adjusted as a concept — you cannot ask for the same block, looser.
- **Pātāl.** Nothing; a `Measurement` is a bare name and value with no distinction between
  a body measurement and a garment one. Note this is a different construct from P-11: P-11
  is surplus length eased into a seam, P-28 is added room in the garment.
- **Ask.** Is ease a named quantity in the file, or arithmetic already folded into the
  formulas?
- **Prior.** **DEFER** · additive, conditional on P-25 · Ease without parametric expressions
  has nothing to modify. It is listed so that P-25's eventual design remembers it, and so
  that the terminological collision with P-11 is recorded once rather than discovered twice.

---

## F — Document and application scope

Rows that are real constructs but probably not schema. They are here so K6 can dismiss them
with a reason rather than leave them unlisted, and so the census is complete.

### P-29 · Display units
- **Is.** mm, cm, or inches — including fractional inches, which is how the entire US home
  sewing market reads a pattern (`5/8"` seam allowance is the near-universal default there).
- **Pātāl.** Millimetres everywhere internally, which is correct and should not change. No
  display preference exists.
- **Prior.** **OUT OF SCHEMA** for the geometry; **FOLD IN** as a document-level display
  preference · additive · One enum on `Project`. Storage stays mm; only presentation
  changes. Worth one field because unit preference is a property of the document's author,
  and losing it on every open is the kind of small daily friction that ends up in a review.

### P-30 · Flatten tolerance
- **Pātāl.** v2 planned: `Project.flatten_tolerance_mm`, private, validated, default 0.01mm
  (§3.5). Already load-bearing for export — §3.8b makes the printed cut line and the saved
  one the same claim by routing both through the project tolerance.
- **Prior.** **IN-V2** · additive · already decided.

### P-31 · Layers — construction versus final
- **Is.** Separating the scaffolding of a draft from the pattern it produces.
- **Pātāl.** Nothing; also nothing to separate, since there is no construction geometry
  (P-25) to hide.
- **Prior.** **DEFER** · additive · Meaningless before P-06 and P-25 exist. Revisit with
  whichever lands first.

### P-32 · Edit history and reversibility
- **Is.** A record of what was done, not just what resulted. The memorandum promises "every
  creative decision should remain editable, interconnected, and reversible".
- **Pātāl.** Nothing.
- **Ask.** Same question as P-12, from the document's side: does the saved file carry
  history, or only the current state?
- **Prior.** **OUT OF SCHEMA** for v2 · none · Undo is an application concern until someone
  wants it to survive a save, at which point it is a document concern and a large one. The
  useful K6 output is not a verdict but an observation: whether either incumbent persists
  history at all.

### P-33 · Lay plan / marker
- **Pātāl.** Nothing. Export gives each piece its own tile grid at its own origin; ADR-008
  records that as a deliberate non-decision, and nesting is a [TODOS](../../TODOS.md) P2
  item depending on P-15 landing.
- **Prior.** **OUT OF SCHEMA** · none · Already tracked, already reasoned about, and it is
  an output-side packing problem rather than a document field.

### P-34 · Assembly instructions
- **Is.** The sewing order and method — what a commercial pattern ships as an instruction
  sheet.
- **Pātāl.** Nothing.
- **Ask.** Does either tool generate instructions, and if so from what? Instructions
  generated from a model imply the model knows P-08 (which edges join) and P-11 (how).
- **Prior.** **OUT OF SCHEMA** · none · But the *ask* is worth keeping, because generated
  instructions would be strong evidence that the tool models seam pairing as data — which
  makes this row a cheap secondary probe for P-08, the row that matters most.

---

## The census

34 rows. `Seamly2D` and `Freesewing` are **empty on purpose** — they are what K5 fills in,
and a value appearing in them before tool contact means this document has been contaminated.

| ID | Construct | ⚠️ | Pātāl today | Seamly2D | Freesewing | Prior verdict | Blast radius |
|---|---|---|---|---|---|---|---|
| P-01 | Authored outline | ⚠️ | v2 planned | | | IN-V2 | structural |
| P-02 | Seam vs cut line authority | ⚠️ | v1, undocumented | | | FOLD IN (as invariant) | none |
| P-03 | **Per-edge seam allowance** | ⚠️ | one scalar/piece | | | **FOLD IN** | **structural** |
| P-04 | Allowance corner treatment | ⚠️ | engine default only | | | DEFER | additive |
| P-05 | Fold edge / cut on fold | ⚠️ | absent | | | **FOLD IN** | **structural** |
| P-06 | Internal construction lines | | absent | | | FOLD IN | additive |
| P-07 | Interior cut-outs | | impossible | | | DEFER | structural |
| P-08 | **Edge identity / seam pairing** | | absent | | | **OPEN** | **structural** |
| P-09 | **Dart** | ⚠️ | absent | | | **OPEN** | **structural** |
| P-10 | Pleat and tuck | ⚠️ | absent | | | DEFER | additive |
| P-11 | Ease / gathering on an edge | | absent | | | DEFER | additive† |
| P-12 | Shape operations (pivot, slash) | | absent | | | OPEN | none† |
| P-13 | **Notch** | ⚠️ | absent | | | **FOLD IN** | **structural** |
| P-14 | Drill hole / punch mark | ⚠️ | absent | | | FOLD IN | additive |
| P-15 | Grain line | | v2 planned | | | IN-V2 | additive |
| P-16 | Nap / directional layout | | absent | | | FOLD IN (on `Material`) | additive |
| P-17 | Cut quantity and handedness | ⚠️ | absent | | | FOLD IN | additive |
| P-18 | Piece identity | | v2 planned | | | IN-V2 | additive |
| P-19 | Piece name | | v1 | | | — | none |
| P-20 | Piece role (lining, facing…) | | absent | | | DEFER | additive |
| P-21 | **Material per cut instance** | | one per piece | | | **OPEN** | **structural** |
| P-22 | Annotation | | absent | | | FOLD IN | additive |
| P-23 | Printed title block | | partial (export) | | | OUT OF SCHEMA | none |
| P-24 | Measurement set as a document | | flat list in project | | | OPEN | structural |
| P-25 | **Parametric expression** | | absent | | | DEFER (size, not merit) | structural |
| P-26 | Constraint / dependency graph | | absent | | | DEFER | none |
| P-27 | Grade rules / size run | | absent | | | DEFER | structural |
| P-28 | Ease (design / wearing) | | absent | | | DEFER | additive† |
| P-29 | Display units | | mm only | | | FOLD IN (preference) | additive |
| P-30 | Flatten tolerance | | v2 planned | | | IN-V2 | additive |
| P-31 | Layers | | absent | | | DEFER | additive |
| P-32 | Edit history | | absent | | | OUT OF SCHEMA | none |
| P-33 | Lay plan / marker | | absent (TODOS P2) | | | OUT OF SCHEMA | none |
| P-34 | Assembly instructions | | absent | | | OUT OF SCHEMA | none |

† conditional on another row landing first — P-11 and P-28 on P-08 and P-25 respectively;
P-12 becomes structural only if operations turn out to be persisted.

**Tally.** 4 already in v2 · 10 fold in · 10 defer · 5 open · 4 out of schema · 1 present.
Twenty-nine of the thirty-four are absent from the format today.

## What F1 actually has to decide ⛔

The v2 freeze is the wave's one-way door, and the point of this list is to shrink what has
to be right at it. Of 34 rows, **three decisions** touch it. Everything else is additive and
can land at v3 for the same price as v2 — by the rule pinned at the top, none of it may hold
the door.

**Decision 1 — the edge-attribute container. Four rows, one answer. TAKEN 2026-08-15.**
P-03 (per-edge allowance), P-05 (fold edge), P-13 (notch position) and P-08's requirement
that an edge be *stably addressable* all reduce to the same question: how does an edge carry
attributes, and how is a position on an edge named? v2 answered it once implicitly, by adding
`joins` as an array parallel to `segments`. Answering it deliberately costs nothing now and
is a breaking migration of the document's core type later — and it needs no evidence from
either incumbent, which made it the one thing K2 could settle on its own.

**Settled:** [the SeamPath blueprint's §3.2](../plans/2026-08-13-seampath-storage-ultraplan.md)
was amended to `edges: Vec<Edge>`, with `Edge` carrying `join` and nothing else. F1 inherits
this rather than deciding it. Two things did *not* change and are worth restating, because the
amendment is easy to over-read: **no attribute was added** — P-03, P-05 and P-13 remain
unresolved and remain K6's, on evidence — and **the container has not been through the
blueprint's own §6 risk pass**, which verdicted APPROVED on the superseded shape. It is
scheduled before step 2 of the build order and should be treated as unreviewed until then.

**Decision 2 — is a dart an object? (P-09)** If yes, the authored outline stops being
authoritative and `cut_boundary()` becomes a function of outline *and* darts. K3 produces
the evidence as a side effect of drafting a bodice block, which cannot be done without a
bust dart. Do not decide this before K3.

**Decision 3 — does material belong to the piece or to the cut? (P-21, with P-17 and P-20)**
If a piece can be cut more than once, `PatternPiece.material` is demoted from a field to a
default and a `CutInstruction` entity appears. A bodice block is unlined, so this wave may
produce no evidence at all — in which case K6's honest output is "undecided, here is what
would settle it", not a verdict formed on taste.

Everything else — P-06, P-14, P-16, P-17, P-22, P-29 — is a fold-in that costs one optional
field and blocks nothing.

**Four other rows are marked structural and still do not gate the freeze**, which is worth
saying explicitly so the discrepancy is not read as an oversight. P-07 (interior cut-outs),
P-24 (measurement set as a document), P-25 (parametric expression) and P-27 (grade rules)
are structural in the sense that they will one day break something — but none of them is a
change to the shape v2 is freezing. They are a *different document model*, arriving whole or
not at all, and no version of v2 can be shaped to accommodate them cheaply. They mean v3
will exist. They are not a reason to keep the door open, because holding it open buys
nothing that closing it forecloses.

## How to answer the persistence question

The question this list exists to ask is *persist or draw*, and it has a different shape in
each tool. Both protocols must be run against the same bodice block from K3/K4.

**Seamly2D — the diff is the evidence.** The project file is inspectable text. So:

1. Save a baseline with the construct absent.
2. Add exactly *one* instance of the construct. Nothing else.
3. Save to a second file and diff the two.

A new node means persisted, and the diff names the element and its attributes for free — a
citation rather than an impression. No new node means the construct was drawn, or derived,
or lives only in the renderer. Record the element name verbatim; K6 needs it and the DXF
layer inventory (§3.3 / K5) will be read against it.

**Freesewing — the core API is the evidence.** A pattern is a program, so everything a
designer writes is trivially "persisted" and the question does not transfer. Its equivalent
is: **does the core library model the construct, or does every pattern re-implement it?**
Search the core package's public API and its documentation for the construct. A named macro,
snippet or method means the system models it; nothing means each designer draws it by hand,
which is the same answer as "drawn" for Seamly2D purposes. Second probe: does the rendered
SVG carry the construct identifiably, or only as anonymous paths?

**Evidence standard, for both.** A row is answered by *tool + version + the exact element,
file path or API name + a one-line quote of what was found*. "Yes, Seamly2D has darts" is
not an answer to any question this document asks — the question is whether the dart is in
the file and what it is called there.

**Absence is a finding and gets the same rigour.** "Searched the saved file for `notch`,
`nadsechka`, and every element added between baseline and second save; not present" is a
result. "Could not find it" is not.

## Pre-registered guesses about the incumbents

Quarantined here, deliberately, and **none of it is evidence**. It is recorded so that being
wrong is visible afterwards rather than silently absorbed — the same discipline the wave
applies to declaring the ±0.5mm tolerance before measuring rather than after.

| # | Guess | Confidence |
|---|---|---|
| G1 | Seamly2D's project file is XML and inspectable in a text editor, so the diff protocol works as written | high |
| G2 | Seamly2D keeps measurements in a separate file from the pattern (P-24) | medium |
| G3 | Seamly2D persists notches and grain lines as named entities (P-13, P-15) | medium |
| G4 | Both tools store construction as formulas over measurements rather than coordinates (P-25) | high |
| G5 | Freesewing's core exposes named helpers for title, grain line, cut-on-fold and notches | medium |
| G6 | **Neither tool models seam pairing as first-class data (P-08)** | low — and this is the guess most worth being wrong about |
| G7 | Both are dependency-DAG rather than bidirectional-solver systems (P-26) | medium |
| G8 | Neither persists an operation history (P-12, P-32) | medium |

If the protocol above cannot be run — G1 is wrong, or K1's go/no-go fails and Seamly2D does
not install on this machine — then this document still stands as the domain census and
Decision 1 above is still decidable. That is the second benefit of the wave's DAG re-cut:
the knowledge track can fail without stranding anything else.

## Filling this in

K5 and K6 edit this file rather than replacing it. The rules that keep it honest:

1. **Do not edit the prior verdicts.** They are the pre-registration. Add a `Verdict (K6)`
   column beside them so the two can be compared, and let the rows where the evidence moved
   the answer be visible.
2. **Every filled cell carries a citation** to the standard above.
3. **Every K6 verdict is FOLD IN or DEFER, and every DEFER carries a reason.** OPEN and
   OUT OF SCHEMA are K2 states; they do not survive into §3.4's output, which is the
   wave's acceptance criterion. A defer with no reason is the schema v3 the SeamPath
   blueprint's D4 warns about.
4. **Score the guesses.** One line: how many of G1–G8 survived. If all eight did, that is
   worth suspecting rather than celebrating — it more likely means the read was shallow
   than that the priors were excellent.
5. **ADR-006 is not written from this table.** The table is the primitive inventory (§3.4).
   The wedge (§3.5) comes from the K3/K4 friction logs — from where the incumbents annoyed
   you while you did real work. A wedge argued from a feature census is the failure mode
   ADR-006 has been held back through two waves to avoid, and this document is a census.






