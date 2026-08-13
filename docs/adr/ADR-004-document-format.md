---
id: ADR-004
title: Document Format — schema version and material identity
status: accepted
date: 2026-08-12
tags: [adr, document, format]
---

# ADR-004 — Document Format

## Status
**Accepted** — 2026-08-12.

## Context

A `.patal` file holds a project: its pieces, their outlines and seam allowances, its
measurements, and the materials those pieces are cut from. No such file has ever been
written by a user.

Two problems existed in the model before this decision.

**Materials had no identity.** `PatternPiece.material` was an `Option<Material>` — an
embedded copy. Editing a material in a library left every piece holding a stale
duplicate, which means the shareable studio libraries the memorandum describes would
have silently disagreed with the pieces cut from them, and nothing would have
reported it.

**Documents had no version.** A file with no version field cannot be safely read by a
later build, because there is no way to tell what shape it was written in. The cost of
retrofitting one is guessing.

## Decision

### Material identity

`MaterialId`, a UUID, private on `Material` with no setter. A caller can change what a
material *is* but not which material it is, so a reference cannot be invalidated
behind its back.

`PatternPiece.material` becomes `Option<MaterialId>`. `Project` owns a
`MaterialLibrary`. Resolution goes through `Project::material_for`, which
distinguishes two states a caller must not conflate:

- `Ok(None)` — no material assigned yet. Normal while designing.
- `Err(MaterialNotFound { piece, id })` — a reference that does not resolve.

**Never `None` for the second case.** A piece that silently forgets its material is a
piece that gets cut from the wrong cloth, and the person who finds out is the one
holding the scissors. Deserializing a `Project` checks every reference, so a
hand-edited or badly-merged file fails at load rather than at the cutting table.

### Document envelope

```rust
pub struct Document { schema_version: u32, project: Project }
pub const SCHEMA_VERSION: u32 = 1;
```

Separate from `Project` so the version is readable before anything else is
interpreted — a loader that must parse the whole project to discover the version
cannot refuse a version it does not understand. A future version is refused with a
message that says so in words rather than surfacing as a parse error.

`schema_version` is private with no setter, for the same reason `MaterialId` is: a
document's version describes the shape it was written in, so letting a caller assign
it lets them claim a shape they did not produce.

### Wire contract: snake_case

Rust does not rename; Swift maps. `PatternPiece` already emitted `seam_allowance_mm`,
so snake_case was already the de facto contract and this makes it the actual one.
Swift's `Material` gained `CodingKeys` accordingly — before this, the two sides
encoded the same material into documents neither could read.

## What this deliberately is not

**This is not settling the format forever**, and the plan's framing that it was should
not be inherited by anyone reading this later.

That framing came by analogy with the bundle identifier in ADR-002, which genuinely is
permanent because Apple says so. The analogy does not hold. A file format is immutable
only once files exist in *someone else's* hands, and for the foreseeable future the
only holder of every `.patal` file is the person who would write the migration.

Worse, the format would have been frozen while grading, darts, notches, grainlines and
the constraint solver are all explicitly unbuilt. Version 2 is close to certain. That
is precisely the situation `schema_version` exists for — not an argument against
shipping it, and not a reason to treat version 1 as sacred.

The parts kept here are cheap and independently correct on their own merits. The
approval gate the plan attached to them was removed.

## Not decided here: persistence

There is no file I/O in the engine. Reading and writing bytes, atomic replacement, and
what happens to a half-written file are a separate concern from what a document *is*,
and the persistence crate was cut from this wave.

The document format is nonetheless exercised against a real disk today: the
engineering harness (ADR-005) writes a `.patal` file and reads it back, in disposable
code, without committing the engine to a persistence API. When the engine does grow
one, atomic write via temp-file-plus-rename is the intended shape, and the temp file
must be removed on **every** failure path.

## Consequences

- **Positive:** editing a material is immediately true for every piece using it. The
  stale-copy class of bug is gone rather than mitigated.
- **Positive:** a dangling reference is loud at load time.
- **Positive:** Swift and Rust now agree on material identity by construction rather
  than coincidence — Swift's `UUID` was previously an invention with no counterpart.
- **Negative:** removing a material does not clean up references to it, by design. The
  alternative — silently unassigning it from every piece — destroys information the
  designer may want back.
- **Open:** `PatternPiece` has no identity field on the Rust side, while Swift's has a
  `UUID`. That is the remaining half of the identity divergence and the reason
  `PatternPiece`'s document shape is still Swift-to-Swift only.
- **Open:** the format currently stores each piece's flattened `PatternBoundary`, not
  its authored `SeamPath`. A file written today cannot be edited back into curves.
  This should be resolved before any file leaves this machine, and is the most likely
  reason for schema version 2.
