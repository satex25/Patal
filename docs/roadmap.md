---
title: Roadmap
tags: [roadmap]
updated: 2026-08-12
---

# Roadmap — the pillars that are not built

Named so the gap between the memorandum's ambition and the code's reality is visible
rather than implied. Nothing here is behind schedule; a foundation is what exists, and
this is what it is a foundation *for*.

See [status](status.md) for what is actually in flight.

## The two that make it a pattern CAD application

These are absent from every plan written so far, which is the most surprising thing
about the plans.

**Export.** DXF-AAMA/ASTM for factories, tiled PDF at true scale for home printing.
Pure Rust, runs on Windows with no Mac, headlessly testable — and the cheapest
possible route to real validation: print a piece at true scale and hand it to a
pattern maker. Every claim about the geometry is currently a test assertion.

**Grading.** Sizing a pattern up and down a size run. Also pure Rust, also testable
without a Mac. A pattern tool that cannot grade is a drawing tool.

Both are strong candidates to be the next wave.

## The largest one

**The parametric constraint solver.** The memorandum describes patterns as "a living
system composed of interconnected relationships" where an edit propagates. Nothing
propagates today: a `Project` is a list of pieces and a list of measurements with no
relationship between them.

This is a project in its own right, not a feature, and it is the thing that separates
Pātāl from a drawing program with a garment theme. Deliberately excluded from the
current wave's scope.

## Pattern primitives

Darts, notches, grainlines, pleats, facings. None modelled. A `PatternPiece` is an
outline and a seam allowance; a real pattern piece carries construction information a
cutter and a sewer both need.

Worth noting these interact with the format: adding any of them is a likely reason for
document schema version 2.

## Canvas and rendering

Metal, per ADR-001. Nothing built. The Tauri harness draws a piece on an HTML canvas
today, which is a stand-in for looking at geometry, not a rendering pillar.

## Sync

Multi-device. Not started, not specified, and not urgent — there is no second device
to sync to.

## Intelligence

The AI collaborator layer. Deliberately last: it needs something worth acting on
before it has anything to act on.

## Before any of it

**Competitive analysis.** [status](status.md) carries this as the next-but-one action.
Seamly2D/Valentina is free, open source, parametric, roughly thirteen years old, and
already ships DXF-AAMA/ASTM export and tiled PDF printing. Freesewing is
parametric-by-code with a real user base. On the axis Pātāl currently competes on —
draw a polygon, offset a seam allowance — it is behind a free incumbent.

That does not mean the project is wrong. It means the wedge has never been written
down, and the honest way to write it is to draft one bodice block in each of them
first rather than compare feature tables.
