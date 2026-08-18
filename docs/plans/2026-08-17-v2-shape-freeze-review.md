---
title: v2 shape freeze — review dossier
tags: [review, decision, gate]
updated: 2026-08-17
status: awaiting operator sign-off
---

# v2 shape freeze — review dossier

> [!important] What this is for
> The gate before Task 8 asks the operator to sign off a wire format. Last session
> declined to sign on the grounds that **the shape had only ever been read as a
> six-bullet prose summary, and approving a wire format nobody has seen the bytes of
> is not a freeze.** This document is the bytes, plus a mechanical check of each of
> the six claims against them.

**Nothing here is signed.** One finding needs an operator decision before the freeze
is meaningful. See [Finding 1](#finding-1--corner-joins-are-omittable-on-read-but-never-omitted-on-write).

## How to reproduce this in one command

```sh
cmd //c 'scripts\cargo.bat test --package patal-pattern -- --nocapture print_v2_shape'
```

`print_v2_shape` is a temporary review instrument in `engine/crates/pattern/src/lib.rs`.
It prints the document below **and asserts every one of the six claims** against the
serialised bytes. Printing alone would leave the reviewer diffing JSON against prose by
eye, which is how a wrong shape gets waved through. **Delete it at Task 8** — it is a
review instrument, not a regression.

The document it prints is deliberately maximal: two pieces, one cubic edge, one
genuinely tangent `Smooth` join, a grain line, a material reference, a non-default
tolerance, and a second piece left bare so the null cases are visible too.

## The six claims, checked against the bytes

| # | Claim | Verdict |
|---|---|---|
| 1 | A piece stores `outline` (a `SeamPath`) and **never** a polygon | ✅ holds |
| 2 | An edge is `{"geometry": {…}, "join": …}` — nested, not flat | ✅ holds |
| 3 | `join` may be omitted and means `corner`; `geometry` may not be omitted | ⚠️ **holds on read, not on write — Finding 1** |
| 4 | A piece carries `id` (bare UUID string), `grain` (nullable), `seam_allowance_mm`, `material` | ✅ holds |
| 5 | A project carries `flatten_tolerance_mm`, defaulting to 0.01 | ✅ holds — and survives reload rather than defaulting back |
| 6 | Deliberately absent: per-edge seam allowance (P-03), fold edges (P-05), notch anchors (P-13) | ✅ holds |

Claim 5 is checked in the strong form: the instrument writes `0.25`, reloads, and
asserts the reloaded project still reports `0.25`. A document written at the default
`0.01` could not distinguish "persisted" from "defaulted back on load", which is why
the fixture uses a non-default value.

Claim 6 is checked by asserting the three field names are *absent* from an edge. That
is the entire argument for the `Edge` container — each of those primitives should land
later as a field on an existing struct rather than as a schema v3 — and this is the
moment it either holds or does not. It holds.

---

## Finding 1 — corner joins are omittable on read but never omitted on write

**This is the one thing needing a decision.**

`Join::Corner` is documented in `curves.rs` as *"the absence of a claim, which is why it
is the serde default: omitting the key cannot manufacture a claim the coordinates
contradict."*

On **read** that is exactly true — a file with no `join` key loads as a corner, and the
instrument proves it by deserialising one.

On **write**, Pātāl emits `"join": "corner"` on every corner edge, because `Edge` has
`#[serde(default)]` but no `skip_serializing_if`. So every document Pātāl produces
manufactures, on almost every edge, precisely the explicit claim the type's own
rationale says omission exists to avoid.

### What it costs

Measured on the review document itself, and extrapolated:

| | |
|---|---|
| Corner joins in the sample | 7 of 8 edges |
| Document size as written | 3,550 bytes |
| With corner joins omitted | 3,326 bytes |
| **Overhead** | **224 bytes — 6.3% of the document** |
| Extrapolated: 50-piece garment, ~20 edges/piece, ~90% corners | **~28 KB of pure redundancy** |

The proportion gets *worse* on real work, not better: the sample is 87.5% corners
because it was built to show off a cubic, and a real drafted block is closer to all
corners with a handful of curves.

### The three arguments that actually matter

1. **It contradicts the type's stated design.** Either `Corner` is the absence of a
   claim — in which case writing it is wrong — or it is a claim like any other, in
   which case the doc comment and the serde default are both misleading. The two
   positions are coherent; the current state is not.
2. **Pātāl's own output never exercises the default path.** Because the writer always
   emits the key, `#[serde(default)]` is only ever exercised by hand-written fixtures.
   Task 8 writes a migration, and an untested read path is exactly where a migration
   bites.
3. **It is cheapest to decide now.** The fix is one attribute
   (`#[serde(skip_serializing_if = …)]` on `Edge::join`). But it changes every byte of
   every document, so after Task 8 writes the migration and v2 files exist, changing it
   costs another migration. That is the definition of the one-way door this gate exists
   to guard.

### The decision

- **Option A — omit corner joins on write.** Honours the documented rationale, ~6% smaller
  documents, and makes Pātāl's own output exercise the default path. Cost: one attribute,
  plus re-blessing any byte-compared fixture. **Recommended.**
- **Option B — keep writing them, and fix the docs instead.** Defensible: an explicit key
  is unambiguous to a human reading the file, and the format is meant to be reviewable.
  Cost: amend `Join::Corner`'s doc comment so it stops claiming an absence semantics the
  writer does not honour.

Either is a legitimate freeze. **Signing without picking one freezes an inconsistency.**

---

## Finding 2 — a `.patal` saved since Task 1 is in a version limbo

Not a shape defect. A migration hazard Task 8 should be built knowing about.

`SCHEMA_VERSION` is still `1`, and Task 8 is what bumps it to `2`. So a document written
by **today's** build carries `"schema_version": 1` alongside the fully v2 *shape*
(`outline`, nested `Edge`, `grain`, `flatten_tolerance_mm`).

This is reachable, not theoretical: the Tauri harness exposes
`save_demo_document(directory, tolerance_mm)`, which writes a real file to a
caller-supplied directory. Anyone who ran the harness after Task 1 has one.

After Task 8, such a file hits the v1 branch on its version number, and the plan's
strict `TryFrom` — correctly — *rejects* a document carrying the wrong version's
fields. So it fails **loudly**, which is the designed behaviour and much better than a
silent mis-migration. But it fails.

**The implication for Task 8:** the `UnsupportedSchemaVersion` path is not the only way
a v1 load can fail, and the error a user gets for this case should say *"this file was
written by a pre-release build; there is no migration for it"* rather than a generic
shape-mismatch error. Worth a named test.

**The implication now:** if you have any `.patal` file saved from the harness, treat it
as disposable. Do not build a fixture from it.

---

## Also worth seeing, not blocking

- **`Point2` serialises as `{"x": …, "y": …}`.** An array `[x, y]` would roughly halve
  the coordinate bytes, which dominate a real pattern. Against that: the object form is
  self-describing and diffable, which is a stated goal of the format. Raised only so the
  choice is deliberate rather than inherited — it is **not** part of the six claims and
  the freeze does not depend on it.
- **`Material` writes all of its optional fields**, including four empty arrays and three
  nulls, on every material. Materials are few, so the cost is small; noted for symmetry.
- **`schema_version` sits at the top of the document**, before `project`. Key order is
  not guaranteed by JSON and the plan's R6 already refuses to depend on it, so this is
  cosmetic — but it does mean a human opening the file sees the version first.

## What the freeze still does not decide

Confirmed by reading, as the gate requires: **freezing v2 does not decide whether a dart
is an object.** That is Decision 2 of the census, still blocked on K3, and a dart lands
as an *additive* field either way — piece-level if it is an object, or as a derived
operation in the dependency graph if it is not. Neither changes any shape above. **The
freeze is safe to sign while K3 is outstanding.**

## Recommendation

Take **Option A** on Finding 1, then sign. The shape is otherwise exactly what the six
claims describe, and the `Edge` container demonstrably does the job it was added for.

Sign-off belongs in `docs/status.md` under "The next decision", and Task 8's first act
should be deleting `print_v2_shape`.

---

## Appendix — the exact bytes

Verbatim output of `print_v2_shape` on this build. This is the artefact being signed.

```json
{
  "schema_version": 1,
  "project": {
    "name": "Shape Freeze Review",
    "pieces": [
      {
        "id": "ee944b01-08bb-4a8f-a648-d6908eb78d54",
        "name": "Bodice Front",
        "outline": {
          "start": {
            "x": 0.0,
            "y": 0.0
          },
          "edges": [
            {
              "geometry": {
                "kind": "line",
                "to": {
                  "x": 100.0,
                  "y": 0.0
                }
              },
              "join": "corner"
            },
            {
              "geometry": {
                "kind": "cubic",
                "c1": {
                  "x": 150.0,
                  "y": 0.0
                },
                "c2": {
                  "x": 200.0,
                  "y": 50.0
                },
                "to": {
                  "x": 200.0,
                  "y": 100.0
                }
              },
              "join": "smooth"
            },
            {
              "geometry": {
                "kind": "line",
                "to": {
                  "x": 0.0,
                  "y": 100.0
                }
              },
              "join": "corner"
            },
            {
              "geometry": {
                "kind": "line",
                "to": {
                  "x": 0.0,
                  "y": 0.0
                }
              },
              "join": "corner"
            }
          ]
        },
        "seam_allowance_mm": 10.0,
        "material": "900ac2bb-818a-4bc6-a9d5-d319be3e91e0",
        "grain": {
          "angle_deg": 15.0,
          "anchor": {
            "x": 50.0,
            "y": 50.0
          }
        }
      },
      {
        "id": "efdd87eb-78b5-46ed-8649-7b79049fd8d5",
        "name": "Waistband",
        "outline": {
          "start": {
            "x": 0.0,
            "y": 0.0
          },
          "edges": [
            {
              "geometry": {
                "kind": "line",
                "to": {
                  "x": 200.0,
                  "y": 0.0
                }
              },
              "join": "corner"
            },
            {
              "geometry": {
                "kind": "line",
                "to": {
                  "x": 200.0,
                  "y": 200.0
                }
              },
              "join": "corner"
            },
            {
              "geometry": {
                "kind": "line",
                "to": {
                  "x": 0.0,
                  "y": 200.0
                }
              },
              "join": "corner"
            },
            {
              "geometry": {
                "kind": "line",
                "to": {
                  "x": 0.0,
                  "y": 0.0
                }
              },
              "join": "corner"
            }
          ]
        },
        "seam_allowance_mm": 10.0,
        "material": null,
        "grain": null
      }
    ],
    "measurements": [],
    "materials": [
      {
        "id": "900ac2bb-818a-4bc6-a9d5-d319be3e91e0",
        "name": "Wool Suiting",
        "weight_gsm": null,
        "thickness_mm": null,
        "stretch_percent": null,
        "drape": "structured",
        "rigidity": "medium",
        "surface_texture": "",
        "durability_notes": "",
        "layer_compatibility": [],
        "stitch_recommendations": [],
        "reinforcement_requirements": [],
        "manufacturing_considerations": []
      }
    ],
    "flatten_tolerance_mm": 0.25
  }
}
```
