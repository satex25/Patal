<!-- /autoplan restore point: ~/.gstack/projects/patal/main-autoplan-restore-20260813-142648.md -->
# ULTRAPLAN BLUEPRINT — The Wedge and Validation Wave

> Execution-ready plan produced by `/ultraplan`. Sections carry stable IDs (§1-§7)
> so the review loop can target them. Keep this file in sync with every accepted revision.

| Field | Value |
|---|---|
| **Goal (verbatim)** | "ultraplan a tasklist for these next tasks": settle SeamPath storage, competitive read of Seamly2D and Freesewing then ADR-006, position export as the validation path |
| **Slug** | `wedge-and-validation-wave` |
| **Date** | 2026-08-13 |
| **Branch** | `main` @ `5b2ef0a` (clean, in sync with `origin/main`) |
| **Status** | REVIEWED by `/autoplan` 2026-08-13 — approved with the DAG re-cut |
| **Execution route** | Two parallel tracks, see §4 revision 6 |
| **Risk class** | **CUT-PATH** at §3.3, §3.7, §3.8-§3.10. Export emits a line someone cuts cloth along. |

**Relationship to the existing blueprint.** `docs/plans/2026-08-13-seampath-storage-ultraplan.md`
already decomposes the SeamPath question into 12 tasks with an APPROVED risk pass. That
work is **not re-planned here**. It enters this wave as a single node, §3.7, carrying its
own build order and its two approval gates. This blueprint plans what surrounds it and,
per D2, moves one thing in front of it.

---

## §1 — Objective Clarification

**Core goal.** Give Pātāl a written wedge and a physically verified cut line: learn what
the incumbents actually model before freezing the file format, store what the designer
drew, then put a true-scale printed pattern in a pattern maker's hands.

**What changed at the gate.** D2 moved the competitive read in front of the v2 schema
freeze, reversing the order in `docs/status.md`. The argument is the SeamPath plan's own
D4 rationale: "each one deferred is a candidate schema v3 for a field already known to be
wanted." Drafting a real block in Seamly2D is how you find out what is wanted. The freeze
is the only irreversible step in the wave, so it should be the best-informed one.

**Success criteria.**

| # | Criterion | Verified by |
|---|---|---|
| S1 | The wedge is written down | ADR-006 exists, cites drafting experience, not a feature table |
| S2 | The primitive gap is enumerated, not guessed | §3.4 inventory lists every construct both tools model that Pātāl does not, each marked fold-in or defer-with-reason |
| S3 | The v2 freeze is informed | §3.6 records each §3.4 finding as folded into v2 or explicitly deferred |
| S4 | Reference artifacts exist in the formats we intend to emit | A Seamly2D DXF-AAMA export and tiled PDF committed as fixtures |
| S5 | A piece stores what the designer drew | Inherited S1-S8 from the SeamPath blueprint |
| S6 | The geometry is true at true scale | A printed piece measures within tolerance of its nominal dimensions, on paper, with a ruler |
| S7 | A pattern maker has held it | Written feedback from someone who sews, recorded in the wave close-out |
| S8 | One implementation of the cut line | Export consumes `cut_boundary()`. No flattening or offset logic in the export crate. |

**Constraints.** C1-C9 carried verbatim from the SeamPath blueprint. Three added for this wave.

| # | Rule | Source |
|---|---|---|
| C1 | Correct or loud. Never return a plausible-looking number from a fallible op. | `geometry/src/lib.rs` header |
| C2 | `#![forbid(unsafe_code)]` stays in every crate. | all four crates |
| C3 | The core imports no platform UI types. | ADR-001 |
| C4 | The render loop never crosses FFI per frame. | ADR-001 |
| C5 | `Pātāl` in prose and UI, `Patal` in anything a toolchain touches. | ADR-002 |
| C6 | Invariants live in the constructor. Private field, no back door via serde. | `geometry/src/lib.rs` |
| C8 | CI gates: fmt, clippy `-D warnings`, test, `cargo deny`, rustdoc, and the five jobs. | `.github/workflows/ci.yml` |
| C9 | The crate does not invent geometry. | `curves.rs:143-149` |
| **C10** | **Export lives in the Rust core, never in the Tauri harness.** The harness is disposable by ADR-005; an export path inside it would be thrown away or, worse, kept. | ADR-001, ADR-005 |
| **C11** | **Export consumes the kernel's cut line, it does not compute one.** A second implementation of where cloth gets cut is the exact liability the Swift offset kernel was deleted to remove. | last wave's D3 |
| **C12** | **A true-scale claim must be checkable on the artifact itself.** Every tiled PDF carries a calibration square. | this plan, §6 |

**Environment.** Unchanged. Rust 1.97.1 pinned, host `x86_64-pc-windows-msvc`, every cargo
invocation through `scripts\cargo.bat` because Git Bash coreutils `link` shadows MSVC
`link.exe`. No macOS toolchain locally; CI `macos-latest` is the only Swift verification.
Seamly2D and Freesewing both need to run on this Windows machine (see §6, first unverified
assumption).

**Assumptions.**

| Assumption | State |
|---|---|
| The SeamPath blueprint is still accurate against current `main` | **UNVERIFIED.** Its header pins `dc509eb`; HEAD is `5b2ef0a`. Three commits landed since (docs consolidation, two CI bumps). None touched `engine/`, so the risk is low, but §3.6 re-verifies rather than assumes. |
| Seamly2D ships a working Windows build | **UNVERIFIED.** Checked at §3.1 start, before any time is sunk. |
| Freesewing runs without a local toolchain | **UNVERIFIED.** It is parametric-by-code and browser-hosted; assumed usable from Windows. |
| Seamly2D's own DXF-AAMA output is conformant | **UNVERIFIED and load-bearing.** If it is not, the reference file is not an oracle. See §6. |
| ADR-006 is reserved for the competitive wedge | **VERIFIED** `docs/adr/README.md:17` |
| The 953-LOC kernel and its 31 tests stay untouched all wave | **VERIFIED as intent**; enforced as an acceptance criterion |
| A pattern maker is reachable for S7 | **UNVERIFIED.** Operator-owned. If not, S7 degrades to self-measurement and the wave says so. |

**Unknowns.** D1-D4 resolved at the gate. One inherited decision stays open and belongs to
the operator: the Swift mirror-or-delete call at §3.9 of the SeamPath blueprint. It is not
re-asked here; it surfaces as an approval node inside §3.7.

---

## §2 — Domain Mapping

**Problem classification.** Three different kinds of problem wearing one wave. §3.1-§3.5 is
a **knowledge** problem: the answer is not in the repo and cannot be derived from it, only
obtained by using other people's software and writing down what happened. §3.6-§3.7 is a
**data** problem: what a file stores, already decomposed elsewhere. §3.8-§3.10 is an
**output and manufacturing** problem, and the first point in this project's life where
reality, rather than a test assertion, gets a vote. The temporal dependency runs
knowledge to data to output, and D2 made that ordering explicit rather than incidental.

There is no risk or capital domain here, so the SATEX vocabulary this skill normally binds
to does not apply. The structural analogue is **cut-path integrity**: any code whose output
becomes a line someone cuts cloth along gets the scrutiny the constitution reserves for
order routing. §3.3, §3.7, and §3.8-§3.10 qualify.

**Touch-map.**

| Area | Path | Role this wave |
|---|---|---|
| geometry: kernel | `engine/crates/geometry/src/lib.rs` (953 LOC, 31 tests) | **Untouched.** Tests passing unmodified is the evidence. |
| geometry: curves | `engine/crates/geometry/src/curves.rs` (402 LOC) | Changed by §3.7 only, per its own blueprint. |
| pattern | `engine/crates/pattern/src/lib.rs` (654 LOC, 18 tests) | Centre of §3.7. May gain fields from §3.4 findings. |
| materials | `engine/crates/materials/src/lib.rs` | Untouched. |
| ffi | `engine/crates/ffi/src/lib.rs` | Untouched. Verified it never constructs a piece. |
| **export (new)** | `engine/crates/export/` | Created at §3.8. Pure Rust, no platform types, consumes `cut_boundary()`. |
| desktop harness | `apps/desktop/src-tauri/src/lib.rs` | Proof surface only. Gains a "write PDF" command, no export logic. C10. |
| native (Swift) | `apps/native/Sources/PatalKit/` | Touched only by §3.7's inherited mirror-or-delete decision. |
| docs | `docs/adr/`, `docs/plans/`, `docs/status.md` | ADR-006 (§3.5), ADR-007 (inherited), ADR-008 (§3.11). |
| fixtures | `engine/crates/export/tests/fixtures/` | Reference DXF and PDF from §3.3. |

**Load-bearing invariant at risk.** One implementation of the cut line. Export is the second
consumer of `cut_boundary()` ever written, and the first one outside the harness. C11 exists
because the cheapest way to write a PDF emitter is to re-flatten inside it.

---

## §3 — Task Decomposition

> Major to sub to atomic. CUT-PATH flags mark anything whose output reaches cloth.

### §3.0 — Settle line endings (housekeeping, unblocked, 10 minutes)
- **Purpose:** `core.autocrlf=true` with no `.gitattributes` means Obsidian re-saving a
  note shows as a phantom modification. Observed live this session on the SeamPath plan.
- **Inputs:** none. **Outputs:** `.gitattributes`.
- Subtasks:
  - Add `.gitattributes` with `* text=auto eol=lf` and a binary rule for `*.png`, `*.pdf`.
  - `git add --renormalize .`, confirm the tree is clean and the diff is empty.

### §3.1 — Draft a bodice block in Seamly2D
- **Purpose:** the wedge cannot be written from feature tables. It comes from where the
  incumbent annoys you while you do real work.
- **Inputs:** a standard bodice block draft (any published set of measurements).
- **Outputs:** a saved Seamly2D project; a running annoyance log.
- **Depends on:** nothing. Parallel with §3.2.
- Subtasks:
  - Confirm a Windows build exists and installs. If not, STOP and re-scope. This is the
    first gate because everything downstream of §3.1 assumes it.
  - Draft the block: neck, shoulder, armscye, bust dart, waist.
  - Log every point of friction as it happens, timestamped, not from memory afterward.
  - Note specifically: how a curve is authored, how a dart is expressed, how a
    measurement change propagates, and what happens when a constraint cannot be satisfied.

### §3.2 — Draft the same block in Freesewing
- **Purpose:** the parametric-by-code end of the spectrum, and the one with a real user base.
- **Outputs:** the same block expressed as code; the same annoyance log, kept separately.
- **Depends on:** nothing. Parallel with §3.1.
- Subtasks:
  - Get a local or hosted environment running.
  - Express the same block. Note what is pleasant about code as the authoring surface and
    what is intolerable about it.
  - Note how it handles the thing Pātāl claims as its centre: an edit propagating through
    a system of relationships.

### §3.3 — Capture reference export artifacts  ⚠️ CUT-PATH
- **Purpose:** turn export from a spec-reading exercise into a diff against known-good output.
- **Inputs:** the §3.1 Seamly2D project.
- **Outputs:** `bodice-reference.dxf` (AAMA/ASTM) and `bodice-reference.pdf` (tiled),
  committed as fixtures with a provenance note recording tool version and export settings.
- **Depends on:** §3.1.
- **Safety note:** these files become the oracle §3.8 is measured against. An unconformant
  reference silently teaches the wrong format. §6 carries the mitigation.
- Subtasks:
  - Export DXF-AAMA/ASTM. Record every export setting chosen.
  - Export a tiled PDF at true scale. Print one page and measure it before trusting it.
  - Read the DXF as text. AAMA is a layer-and-entity convention over DXF; confirm the
    layer names present match what the standard describes, and write down which ones appear.
  - Commit both with a README recording tool version, settings, and the measured print check.

### §3.4 — Primitive inventory  (the input to the v2 freeze)
- **Purpose:** enumerate what both tools model that Pātāl does not, while the schema is
  still soft. This task is the reason D2 reordered the wave.
- **Inputs:** §3.1, §3.2, §3.3 outputs.
- **Outputs:** a table in ADR-006, and a fold-in/defer verdict per row feeding §3.6.
- **Depends on:** §3.1, §3.2, §3.3.
- Subtasks:
  - List every construct encountered: dart, notch, grainline, seam allowance per edge,
    fold line, pleat, facing, size run, annotation, piece metadata.
  - For each: does it live in the file, or is it drawn? Anything the incumbents persist is
    a candidate v2 field.
  - Mark each **fold into v2** or **defer**, with a reason on every defer. A defer with no
    reason is the schema v3 the SeamPath plan's D4 warns about.

### §3.5 — Write ADR-006, the wedge
- **Purpose:** close the gap `docs/adr/README.md` has carried since it was written.
- **Inputs:** everything from §3.1-§3.4.
- **Outputs:** `docs/adr/ADR-006-competitive-wedge.md`; `docs/adr/README.md` row moved from
  "not yet written" to Accepted.
- **Depends on:** §3.4.
- Subtasks:
  - State honestly where Pātāl is behind. Seamly2D is free, parametric, thirteen years old,
    and ships both export formats today.
  - Name the wedge as something a person would switch for, in one sentence.
  - Record what was rejected, matching the ADR house style where the value is in the
    rejected section.
  - **If the honest conclusion is that no wedge exists yet, write that.** See §6.

### §3.6 — Reconcile the SeamPath blueprint against current main
- **Purpose:** that plan was written at `dc509eb`; HEAD is `5b2ef0a`. It also predates §3.4.
- **Outputs:** a revision-log entry in the SeamPath blueprint; its header re-pinned.
- **Depends on:** §3.4. Blocks §3.7.
- Subtasks:
  - Re-verify its four VERIFIED assumptions against current `engine/` source. The three
    intervening commits touched docs and CI only, so expect no drift, but confirm rather
    than assume.
  - Fold each §3.4 "fold into v2" row into its §3.6/§3.7 task descriptions.
  - Re-pin the header to current HEAD and log the reconciliation.

### §3.7 — Execute the SeamPath storage blueprint  ⚠️ CUT-PATH  ⛔ two approval nodes
- **Purpose:** make the authored curve the thing a `.patal` stores.
- **Inputs:** `docs/plans/2026-08-13-seampath-storage-ultraplan.md`, reconciled by §3.6.
- **Outputs:** its 12-step build order, its deliverables, ADR-007.
- **Depends on:** §3.6.
- **Approval nodes inherited, not re-decided here:**
  - ⛔ **v2 shape freeze** (its build order step 7). One-way door: the migration encodes
    the final shape. Now informed by §3.4.
  - ⛔ **Swift mirror or delete** (its §3.9). Its recommendation is mirror. CI is gated on
    `native`, so leaving it stale is the one answer that is definitely wrong.
- **Safety note:** all inherited CUT-PATH flags and its S1-S8 criteria apply unchanged.

### §3.8 — Tiled PDF export  ⚠️ CUT-PATH
- **Purpose:** the cheapest route to real validation, per `docs/status.md`.
- **Inputs:** a `PatternPiece` post-§3.7; `bodice-reference.pdf` from §3.3.
- **Outputs:** `engine/crates/export/` with a tiled PDF emitter; unit and golden tests.
- **Depends on:** §3.7 (it exports pieces, so the piece shape must be frozen first).
- **Constraints:** C10 (core, not harness), C11 (consume `cut_boundary()`), C12 (calibration square).
- Subtasks:
  - Create the crate. `#![forbid(unsafe_code)]`, no platform types, no UI deps.
  - Decide the PDF strategy: a minimal hand-rolled writer, or a crate. Vector output only.
  - Emit at true scale: 1mm in the model is 1mm on paper. Page size, margin, and overlap
    are parameters with validated ranges, not constants.
  - **Calibration square on every page**, a labelled 50mm box, per C12.
  - Tile registration: overlap marks and page coordinates so pages can be assembled.
  - Golden test: a known piece produces a byte-stable PDF, so regressions are visible.
  - Assertion test: the emitted outline equals `cut_boundary()` output transformed by a
    pure page transform. This is how C11 is enforced mechanically rather than by intent.

### §3.9 — True-scale correctness  ⚠️ CUT-PATH
- **Purpose:** printers scale by default. This is the single most likely way a correct
  geometry engine produces a wrong physical pattern.
- **Depends on:** §3.8.
- Subtasks:
  - Document the print path that preserves scale (100%, no fit-to-page) in `docs/setup/`.
  - Print the calibration square. Measure with a steel rule. Record actual against nominal.
  - If it disagrees, the bug is in the PDF's page geometry, not the printer. Fix and repeat.

### §3.10 — Physical validation  ⚠️ CUT-PATH  ⛔ approval node
- **Purpose:** S6 and S7. The first outside vote this project has ever taken.
- **Depends on:** §3.9.
- **Why an approval node:** this is where the wave either validates the foundation or
  returns a verdict that reorders the roadmap. Do not schedule work past it in advance.
- Subtasks:
  - Print the bodice block at true scale, assemble the tiles, cut it out.
  - Measure against nominal: neck, shoulder, armscye depth, waist.
  - Hand it to someone who sews. Record their feedback verbatim, including anything about
    what is missing rather than wrong: notches, grainline, labels.
  - Write the outcome into `docs/status.md` whether it flatters the project or not.

### §3.11 — ADR-008, export format decisions
- **Depends on:** §3.10.
- Subtasks:
  - Record the PDF strategy chosen and what was rejected.
  - Record the tiling convention, the calibration square, and the true-scale contract.
  - State the DXF-AAMA position: deferred to the next wave, with §3.3's reference file and
    the layer inventory as its starting point.

### §3.12 — Wave close-out
- **Depends on:** §3.11.
- Subtasks:
  - `docs/status.md`: rewrite "Right now" and "Next, in the order I would do it".
  - `docs/roadmap.md`: move export from "not built" to partially built; note grading is now
    the nearest unbuilt pillar.
  - Root `README.md`: add the export crate to the tree; update the Status section honestly.
  - `docs/adr/README.md`: ADR-006, 007, 008 rows.

---

## §4 — Dependency + Ordering (DAG)

> **REVISED by `/autoplan` 2026-08-13 (revision 6).** The original sequence made §3.8 wait on
> §3.7. That edge is one function signature wide: export consumes `cut_boundary()`, which exists
> today at `pattern/src/lib.rs:186` and returns a plain `PatternBoundary`. §3.7 changes the
> signature, not the page transform, tiling, calibration square, or PDF writer. The old order put
> the wave's cheapest and highest-information event — print it, measure it, hand it to a human —
> at step 13 of 16, behind both one-way doors. It now runs against today's v1 piece, in parallel
> with the competitive read. §3.7 stays fully informed by §3.4, because that edge is untouched.

**Ordered execution sequence (revised).**

```
  KNOWLEDGE TRACK   §3.0 → {§3.1 ∥ §3.2} → §3.3 → §3.4 → §3.5 (ADR-006)
                                                        │
  VALIDATION TRACK  §3.8 (export vs v1 piece) → §3.9 → §3.10(⛔)
                                                        │
                            both tracks join ───────────┴──▶ §3.7(⛔⛔) → §3.8b → §3.11 → §3.12
```

- **§3.8 now takes the v1 `PatternPiece`.** `cut_boundary()` today needs no tolerance argument,
  so the export crate can be written, tested, printed and measured before the schema moves.
- **§3.8b (new, small).** After §3.7 lands, update the call site to `Project::cut_boundary(piece)`
  and delete the v1 path. Export passes no tolerance of its own — the persisted project tolerance
  governs both the saved cut line and the printed one, so S5 and S6 stay one claim.
- **§3.4 still gates the v2 freeze.** The re-cut removes the §3.7 → §3.8 edge only. The
  §3.4 → §3.7 edge, which is what D2 was actually protecting, is unchanged.
- **§3.6 is folded into §3.7 as its first step** (audit-trail decision 12): a node whose expected
  output is "no drift" is a five-minute check, not a DAG node blocking the wave's largest task.

**Superseded sequence, for the record:**
§3.0 → {§3.1 ∥ §3.2} → §3.3 → §3.4 → §3.5 → §3.6 → §3.7(⛔⛔) → §3.8 → §3.9 → §3.10(⛔) → §3.11 → §3.12

**Parallelizable set.** { §3.1, §3.2 } have no mutual dependency. §3.0 is unblocked and can
run at any point. §3.5 (writing ADR-006) can overlap §3.7 once §3.4 is done, since §3.7
consumes the §3.4 table rather than the finished ADR. **The knowledge track and the validation
track are now independent of each other and can interleave freely** — that is the point of the
re-cut, and it is what lets a stall in one avoid stranding the other.

**Approval nodes (one-way doors, operator sign-off required).**
- ⛔ **§3.7 / v2 shape freeze.** The migration encodes the final shape. Reversing it after
  a file exists in the wild means schema v3.
- ⛔ **§3.7 / Swift mirror or delete.** Inherited. Recommendation is mirror. Not re-asked here.
- ⛔ **§3.10 / physical validation.** Not a door you walk through so much as a verdict you
  receive. Plan nothing past it in detail.

**Hard gate.** §3.1's first subtask is a go/no-go: if Seamly2D does not run on this machine,
§3.3 and much of §3.4 lose their source and the wave re-scopes before time is spent.

```
  §3.0 (free)

  KNOWLEDGE TRACK
  §3.1 ──┐
         ├──▶ §3.3(⚠️) ──▶ §3.4 ──┬──▶ §3.5 (ADR-006)
  §3.2 ──┘                        │
                                  └──────────────────┐
                                                     │
  VALIDATION TRACK  (independent — starts immediately, needs only the v1 piece)
  §3.8(⚠️) ──▶ §3.9(⚠️) ──▶ §3.10(⚠️⛔) ────────────┤
                                                     │
                                                     ▼
                                  §3.7(⚠️⛔⛔) ──▶ §3.8b ──▶ §3.11 ──▶ §3.12
                                  (§3.6 folded in as step 1)
```

---

## §5 — Execution Specification

### §5.1 — spec for §3.1 and §3.2 (the drafting)
- **Method:** structured self-observation. Draft the same artifact in both tools and keep a
  contemporaneous friction log. The log is the deliverable; the block is the excuse to produce it.
- **Expected artifacts:** two saved projects, two friction logs with timestamps.
- **Validation:** a neutral reader can name three concrete things each tool does better than
  the other. If the log cannot support that, the read was too shallow.
- **Failure modes:** the tools are hard enough that time goes into learning rather than
  observing; the block drafted is too simple to hit any interesting constraint.
- **Fallback:** timebox each tool. If the block will not come together, a partial draft plus
  an honest log of where it stalled is still a valid input to §3.4. Mastery is not the goal.

### §5.2 — spec for §3.3 (reference artifacts)  ⚠️ CUT-PATH
- **Method:** export, then verify the export rather than trusting it. Read the DXF as text;
  print the PDF and measure it.
- **Expected artifacts:** two fixtures plus a provenance README.
- **Validation:** the printed reference measures true at 100% scale, and the DXF's layer
  names are written down and match the AAMA convention as described.
- **Failure modes:** Seamly2D's AAMA output is non-conformant or partial, so the oracle is
  wrong. A tiled PDF that is not actually true-scale, discovered later.
- **Fallback:** if the DXF cannot be trusted, demote it from oracle to sample and mark
  DXF-AAMA as spec-driven in ADR-008. Say so explicitly rather than quietly relying on it.

### §5.3 — spec for §3.5 (ADR-006)
- **Method:** existing ADR house style. Frontmatter `id`/`title`/`status`/`date`, then
  Context, Decision, Consequences, with the rejected alternatives carrying real weight.
- **Expected artifacts:** `docs/adr/ADR-006-competitive-wedge.md`; README row updated.
- **Validation:** the wedge is stated as one sentence a person would switch for, and the
  document cites specific drafting experience rather than feature comparison.
- **Failure modes:** the ADR becomes a feature table, which `docs/adr/README.md` explicitly
  warns against. Or it overstates the wedge to justify the project.
- **Fallback:** "no wedge identified yet, here is what would have to be true" is a valid and
  publishable ADR outcome. See §6.

### §5.4 — spec for §3.8 (tiled PDF)  ⚠️ CUT-PATH
- **Method:** a new `patal-export` crate consuming `cut_boundary()`. Page layout is a pure
  transform: model millimetres to page points, then tile. No geometry decisions inside export.
- **Expected artifacts:** the crate, a golden-file test, a `cut_boundary()` equality
  assertion, a harness command that writes a file.
- **Validation:** all four CI gates plus the two new tests. C11 is enforced by the equality
  assertion, not by review.
- **Failure modes:** re-implementing flattening inside the emitter (C11 breach); hard-coded
  page constants that silently break A4 vs Letter; a scale error that only shows on paper.
- **Fallback:** if a PDF crate proves heavy or drags in unwanted deps, hand-roll a minimal
  vector writer. The output surface needed here is small: lines, cubics, text labels.

### §5.5 — spec for §3.10 (physical validation)  ⚠️ CUT-PATH
- **Method:** print, assemble, cut, measure, then hand it to a human who sews.
- **Expected artifacts:** recorded measurements, actual against nominal; verbatim feedback;
  a status entry.
- **Validation:** measured dimensions fall within a stated tolerance, declared in advance
  rather than after seeing the numbers.
- **Failure modes:** printer scaling defeats the test; tiles do not register; the reviewer
  gives politeness instead of criticism.
- **Fallback:** if no pattern maker is reachable, S7 degrades to self-measurement and the
  wave records that the outside vote is still outstanding. Do not silently drop it.

---

## §6 — Risk + Ambiguity Audit (self-adversarial)

### CRITIC pass

**Assumptions not verified.**
- **Seamly2D runs on this Windows machine.** Everything from §3.3 onward depends on it.
  Mitigated by making it §3.1's first subtask and a hard gate, so failure is cheap.
- **Seamly2D's DXF-AAMA output is conformant.** This is the sharpest unverified assumption
  in the plan, because a wrong oracle teaches a wrong format convincingly. §5.2 handles it
  by reading the DXF as text and demoting it to a sample if it does not match the described
  convention. DXF is deferred to a later wave anyway, which bounds the damage.
- **The SeamPath blueprint still applies at `5b2ef0a`.** Low risk, three doc/CI commits, but
  §3.6 verifies rather than assumes.
- **A pattern maker is reachable.** Operator-owned. §5.5 states the degraded path.

**Worst case if wrong.** The competitive read concludes Seamly2D does everything Pātāl
plans, better and for free. That is a real possible outcome and the plan must survive it
rather than be structured to avoid hearing it. If it happens, ADR-006 says so, and the
project's honest options are to find a narrower wedge or to stop. A plan that cannot return
that verdict is not a competitive analysis, it is a justification exercise.

**Left out, now added.**
- **Line endings.** Added as §3.0 after this session observed Obsidian producing phantom
  modifications under `core.autocrlf=true` with no `.gitattributes`.
- **The harness accumulates.** Every wave adds a command to `apps/desktop`. ADR-005 calls it
  disposable, but nothing schedules disposal. §3.8 keeps export logic out of it (C10); a
  future wave should decide when the harness gets pruned. Named here, not solved here.
- **Fixture size.** Committed PDFs and DXFs are binary and grow the repo permanently. §3.3
  should commit one reference of each, not a set.
- **The v2 freeze can still be wrong even when informed.** §3.4 reduces that risk, it does
  not remove it. The inherited D3 decision to build a real migration is what makes being
  wrong survivable, and it stays.

### RISK-AGENT pass (rebound to Pātāl's rules)

The SATEX rules this skill normally checks (1% per trade, live-capital actions, risk-param
self-modification, single-signal trade logic) have no analogue in a pattern CAD tool. The
binding is re-pointed to Pātāl's own constraints and to cut-path integrity.

| Rule | Check | Verdict |
|---|---|---|
| C1 correct-or-loud | Export page params validated; true-scale claim checkable on the artifact via C12. | PASS |
| C2 no unsafe | The export crate declares `#![forbid(unsafe_code)]`. | PASS |
| C3 core purity | `patal-export` takes geometry and emits bytes. No platform or UI types. | PASS |
| C8 CI gates | New crate joins the workspace, so all five jobs cover it. | PASS |
| C10 export in core | §3.8 creates `engine/crates/export/`. Harness gets a call site only. | PASS (after revision 1) |
| C11 one cut line | Enforced by the `cut_boundary()` equality assertion, not by intent. | PASS (after revision 2) |
| Cut-path integrity | The 953-LOC kernel is untouched all wave. Its 31 tests pass unmodified. | PASS |
| Kernel purity | No task modifies `geometry/src/lib.rs`. | PASS |

**Verdict: APPROVED, after two forced revisions.**

**Revision 1, VETOED then fixed.** An earlier draft put the tiled PDF writer in the Tauri
harness, reasoning that the harness is where files already get written and it is the only
thing that runs on this machine. That breaches ADR-005: the harness is explicitly
non-shipping and disposable. Export would have been either thrown away or, worse, kept and
promoted by accident. Replaced with `engine/crates/export/`, and the rule written down as C10.

**Revision 2, VETOED then fixed.** An earlier §3.8 had the emitter call `flatten()` itself to
get points to draw, which is the obvious implementation. That reintroduces exactly the defect
class the last wave removed by deleting the Swift offset kernel: two pieces of code deciding
where cloth gets cut. Replaced with a hard requirement to consume `cut_boundary()`, plus a
test asserting the emitted outline equals it under a pure page transform. Written down as C11.

**Unresolved item surfaced to the operator.** Whether a pattern maker is reachable for S7.
It changes what §3.10 can claim, and it is scheduling, not engineering, so it is not a
decision brief. Flagged for the operator to answer before §3.10 is reached.

---

## §7 — Final Assembly: the plan

**Build order.**

> **REVISED (revision 6).** Two tracks now run independently. Steps V1-V6 need only today's v1
> piece; steps K1-K6 are the competitive read. They join at F1.

**Step 0, before either track.** Find a pattern maker for S7 and get a yes. If no one is reachable
by the time V5 is ready, S7 degrades to self-measurement and the wave records that the outside vote
is still outstanding (§5.5). This moved to the front because at step 14 it was too late to go looking.

**Step 0b.** Declare the tolerance: **±0.5mm over 200mm**. In advance, per §5.5, and before any
measuring happens.

**KNOWLEDGE TRACK**

- **K0.** §3.0 `.gitattributes`, renormalize. → `git status` clean after an Obsidian save.
- **K1. Go/no-go:** confirm Seamly2D installs on Windows. → if no, the knowledge track re-scopes.
  The validation track is unaffected, which is the second benefit of the re-cut.
- **K2.** Draft the §3.4 primitive candidate list from domain knowledge first (decision 11). The
  tools then answer only the question a textbook cannot: does the incumbent *persist* it, or draw it?
- **K3.** §3.1 draft the block in Seamly2D, against the pre-registered friction questions (decision 4).
- **K4.** §3.2 same block in Freesewing (parallel with K3). → second friction log.
- **K5.** §3.3 capture the reference DXF and tiled PDF. **Conformance verification dropped**
  (decision 10) — the DXF is a sample, not an oracle, and ADR-008 says so.
- **K6.** §3.4 inventory → §3.5 ADR-006. Includes the vector-editor axis (decision 13) and, per the
  review, the pre-committed statement of which verdicts halt §3.7 and which do not.

**VALIDATION TRACK** (starts immediately, in parallel with K)

- **V1.** Create `engine/crates/export/` **and add `"crates/export"` to `engine/Cargo.toml` members**
  (E-1 — without this all five CI jobs stay green while the crate is never compiled).
- **V2.** Hand-roll the PDF writer, ~350 LOC, zero new dependencies (decision 21). `Mm`/`Pt`
  newtypes and one `MM_PER_PT` so a raw `f64` cannot reach the page transform (X-2). Stroke width
  pinned in points, hairline (F10). No `scale` parameter, ever (X-7).
- **V3.** `ExportError` with named variants; validate page params in the constructor; emit
  all-or-nothing (F4/F5/F6). Return `Result<Vec<u8>, ExportError>`; the harness owns temp-file +
  rename (F3). Takes a **slice** of pieces, not one (X-6).
- **V4.** Tests: absolute scale (50mm = 141.7322835pt, computed without the emitter's transform),
  tile reassembly from registration marks, semantic golden, error paths. See the test plan artifact.
- **V5.** §3.9 true-scale on paper: 200mm ruled line on **both** axes plus the 50mm square (X-4),
  measured on **two** printers (F11), with printer model and driver recorded (E2). Driver paper size
  must equal the PDF page size (X-1). Write `docs/setup/printing.md` (X-9).
- **V6.** ⛔ §3.10 print, assemble, cut, measure, hand it over. → recorded verdict.

**JOIN**

- **F1.** ⛔ **v2 shape freeze.** Operator sign-off, informed by K6 and now also by V6's verdict.
- **F2.** ⛔ **Swift mirror or delete.** Inherited; recommendation is mirror.
- **F3.** §3.7 execute the SeamPath blueprint's 12 steps, with §3.6's reconciliation as its step 1
  (decision 12). → its S1-S8 green, ADR-007.
- **F4.** §3.8b update export's call site to `Project::cut_boundary(piece)`; delete the v1 path.
- **F5.** §3.11 ADR-008 → PDF decisions, tiling and assembly convention, the stroke-centre rule,
  the DXF position as sample-not-oracle.
- **F6.** §3.12 close-out: status, roadmap, README, ADR index.

**Acceptance criteria.**

- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings` clean
- [ ] `cargo test --workspace --locked` green, count reported and above the post-§3.7 baseline
- [ ] `cargo deny check` clean
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` clean
- [ ] All five CI jobs green, including `native`
- [ ] All 31 pre-existing `geometry/src/lib.rs` tests pass **unmodified**
- [ ] Inherited S1-S8 from the SeamPath blueprint all green
- [ ] ADR-006 exists; `docs/adr/README.md` no longer lists it as unwritten
- [ ] Every §3.4 row is marked fold-in or defer, and every defer carries a reason
- [ ] Reference DXF and tiled PDF committed with provenance
- [ ] Export outline equals `cut_boundary()` under a pure page transform (C11, mechanical)
- [ ] Every emitted PDF page carries a calibration square (C12)
- [ ] A printed calibration square measures 50mm with a steel rule
- [ ] A printed bodice block measured against nominal, numbers recorded
- [ ] `docs/status.md` records the physical verdict, flattering or not

**Deliverables.** This blueprint; ADR-006; ADR-007 (via §3.7); ADR-008; `patal-export`;
two reference fixtures; `.gitattributes`; a printed and measured pattern piece; refreshed
status, roadmap, README, and ADR index.

---

## Decision Log

| D# | Question | Chosen | Why |
|---|---|---|---|
| D1 | Boundary confirm | **Draft it** | Scope: sequence the three efforts, decompose the unplanned work, wrap the existing SeamPath blueprint rather than re-plan it. |
| D2 | Competitive read vs v2 freeze | **Read first** | `roadmap.md` puts competitive analysis "Before any of it", and the SeamPath plan's own D4 argues each deferred field is a candidate schema v3. The freeze is the wave's only irreversible step, so it should be the best-informed one. |
| D3 | Depth of the competitive read | **Draft + export from both** | Exporting from Seamly2D yields reference files in the exact formats Pātāl intends to emit, converting export from spec-reading into diff-against-known-good. |
| D4 | First export deliverable | **Tiled PDF** | `status.md` calls it the cheapest route to real validation. Pure Rust, Windows-testable, no Mac, and it cannot be faked by a passing test suite. |

**Inherited, still open:** Swift mirror or delete (SeamPath blueprint §3.9). Surfaces as an
approval node at build-order step 10. Not re-asked here.

## Revision Log (review loop)

| # | Section | Change | Trigger |
|---|---|---|---|
| 1 | §3.8, C10 | Export moved from the Tauri harness into `engine/crates/export/` | §6 RISK-AGENT veto on ADR-005 |
| 2 | §3.8, C11 | Emitter must consume `cut_boundary()`; equality assertion added | §6 RISK-AGENT veto on one-cut-line |
| 3 | §3.0 | Added `.gitattributes` task | CRITIC pass: phantom modifications observed live this session |
| 4 | §3.4 | Every defer must carry a reason | CRITIC pass: an unreasoned defer is the schema v3 D4 warns about |
| 5 | §3.5 | "No wedge identified" made an explicitly valid outcome | CRITIC pass: a plan that cannot return that verdict is a justification exercise |
| 6 | §4, §7 | **DAG re-cut: export + print + measure no longer waits on the v2 freeze.** Two independent tracks. | `/autoplan`: the §3.7→§3.8 edge is one function signature wide; validation was scheduled last, behind both one-way doors |
| 7 | §7 | Pattern-maker hunt + tolerance declaration moved to step 0 | `/autoplan` premise gate: S7 is unverified and outside operator control; no tolerance number existed anywhere in 513 lines |

---

# /autoplan REVIEW OUTPUTS

> Appended by `/autoplan` on 2026-08-13. Restore point:
> `~/.gstack/projects/patal/main-autoplan-restore-20260813-142648.md`.
> Voices: Claude subagent only (`[subagent-only]` — Codex CLI not installed).

## What already exists (CEO 0B)

| Sub-problem | Existing code | Reused? |
|---|---|---|
| Store the authored curve | `SeamPath`/`EdgeSegment`, `geometry/src/curves.rs` (402 LOC) | Yes, via SeamPath §3.1 |
| Offset-aware flattening | `SeamPath::flatten_for_offset`, `curves.rs:285` | Yes — written last wave, still unused |
| The cut line | `PatternPiece::cut_boundary`, `pattern/src/lib.rs:186` → `PatternBoundary::offset`, `geometry/src/lib.rs:357` | Yes, enforced by C11 |
| Schema versioning | `SCHEMA_VERSION`, `pattern/src/lib.rs:30`; `UnsupportedSchemaVersion` | Yes |
| Piece identity template | `MaterialId`, `materials` crate | Yes, copied by SeamPath §3.4 |
| A file-writing surface | `save_demo_document`, `apps/desktop/src-tauri/src/lib.rs` | Yes — call site only, per C10 |
| CI coverage for a new crate | workspace jobs, `.github/workflows/ci.yml` | Automatic on workspace membership |
| PDF emission, page transform, tiling | **nothing** | **New. The only genuinely new engineering in the wave.** |

Nothing is rebuilt. C11 is what prevents the one rebuild that would otherwise happen by default.

## NOT in scope (deferred, with rationale)

| Item | Why deferred |
|---|---|
| DXF-AAMA emitter | §3.11 already defers it; only the reference capture stays |
| Grading / size runs | Separate pillar per `roadmap.md`; no dependency either way |
| Parametric constraint solver | A project in its own right; explicitly excluded |
| Multi-piece lay-plan / nesting (E3) | Outside blast radius; needs a packing algorithm, not a page transform |
| Metal canvas | No Mac; unrelated pillar |
| Harness pruning schedule | Named in §6, correctly not solved here |

## Dream state delta

Advances the format and export pillars. Advances the constraint solver — which `roadmap.md`
calls "the thing that separates Pātāl from a drawing program with a garment theme" — by zero.
The wave can therefore close having *written down* a wedge it has not begun to build.
ADR-006 must name which pillar the wedge lives on, so §3.12's roadmap rewrite is forced to
confront the gap rather than restate it.

## Error & Rescue Registry — `patal-export` (CEO §2)

The plan names no error type for the new crate. C1 ("correct or loud") requires each row to be
a named variant, not a panic and not a plausible-looking default.

| Codepath | What can go wrong | Exception class | Rescued? | User sees |
|---|---|---|---|---|
| `export::tiled_pdf` | piece wider than one page minus margins | `PieceExceedsPageArea { piece, needed_mm, available_mm }` | **N ← GAP** | must not silently clip |
| `export::tiled_pdf` | page size / margin / overlap non-finite or <= 0 | `InvalidPageGeometry { field, value }` | **N ← GAP** | rejected at construction |
| `export::tiled_pdf` | overlap >= page dimension | `OverlapExceedsPage { overlap_mm, page_mm }` | **N ← GAP** | rejected at construction |
| `Project::cut_boundary` (upstream) | offset self-intersects at a sharp corner | `PatternError::Geometry(OffsetSelfIntersects)` | Y (exists) | propagate, name the piece |
| `export::tiled_pdf` | project has zero pieces | `NothingToExport` | **N ← GAP** | refuse; do not write a 0-page PDF |
| write to disk | I/O failure mid-write | `io::Error` | **N ← GAP** | **write to temp + atomic rename; never leave a partial PDF** |

**The partial-file row is the cut-path one.** A half-written PDF that still opens and prints is
exactly the class of defect C1 exists to forbid.

## Failure Modes Registry

| Codepath | Failure mode | Rescued? | Test? | User sees | Logged? |
|---|---|---|---|---|---|
| page transform | scale error (mm→pt wrong by a factor) | N | **only the steel rule catches it** | a wrong-size garment | N — **CRITICAL GAP** |
| PDF write | partial file after I/O error | N | N | a printable, wrong document | N — **CRITICAL GAP** |
| page params | unvalidated size/margin/overlap | N | N | silent A4-vs-Letter breakage | N — **CRITICAL GAP** |
| export tolerance | unspecified which tolerance export flattens at | N | N | visible facets on a curve | N — **CRITICAL GAP** |
| golden test | byte-stable PDF churns on any dep bump | N/A | Y | red CI, then a disabled test | N |
| §3.3 DXF oracle | non-conformant reference teaches a wrong format | Y (§5.2 demotes to sample) | N/A | wrong format convention | N |

**4 CRITICAL GAPS.**

## Architecture (eng §1)

```
  BEFORE                                AFTER
  ┌──────────┐                          ┌──────────┐
  │ geometry │◀──┐                      │ geometry │◀──┬──────────┐
  │  kernel  │   │                      │ UNTOUCHED│   │          │
  └──────────┘   │                      └──────────┘   │          │
       ▲         │                           ▲         │          │
  ┌────┴─────┐ ┌─┴──────┐              ┌─────┴────┐ ┌──┴─────┐ ┌──┴────────┐
  │ pattern  │ │  ffi   │              │ pattern  │ │  ffi   │ │  export   │
  │ 654 LOC  │ │ 142 LOC│              │ +SeamPath│ │UNTOUCH │ │   NEW     │
  └──────────┘ └────────┘              └──────────┘ └────────┘ └───────────┘
       ▲                                    ▲                       ▲
  ┌────┴──────────┐                    ┌────┴───────────────────────┴──┐
  │ desktop       │                    │ desktop (call site only, C10) │
  │ harness       │                    │ + owns temp-file/rename I/O   │
  └───────────────┘                    └───────────────────────────────┘

  export in:   Project::cut_boundary(piece) -> PatternBoundary
  export out:  Result<Vec<u8>, ExportError>   (no std::fs — keeps C3 clean)
  export's dependency on the v2 SHAPE: none. Only on that one signature.
```

Coupling is one-directional, no cycle, C3 holds. **The graph shows the §3.7 → §3.8 DAG edge is
one function signature wide, not a schema freeze wide.**

## Eng findings (Phase 3)

| # | Severity | Finding | Fix |
|---|---|---|---|
| E-1 | **critical** | `engine/Cargo.toml:3-8` lists 4 members; the plan never adds `crates/export`. §6 line 422 claims C8 PASS on workspace membership the plan does not create. An unlisted crate is silently untested by all five green CI jobs. | Add `"crates/export"` to `[workspace] members` as an explicit §3.8 subtask |
| E-2 | high | The plan writes `cut_boundary()` (no args) six times. Post-SeamPath-§3.6 it is `cut_boundary(&self, tolerance_mm)` + `Project::cut_boundary(&self, piece)`. Nothing reconciles *this* plan against that API change. | Export calls `Project::cut_boundary(piece)` and never passes a tolerance |
| E-3 | high | Export's flatten tolerance is unspecified. 0.01mm is invisible on paper; ADR-003's 0.4mm is a visible facet on a neckline. | State the contract: export inherits the persisted project tolerance |
| F1 | **critical** | C11's equality assertion compares the emitter's input to itself. Breach modes are downstream: a re-flatten in the writer, clipping at tile seams, decimal rounding. | Parse the emitted content stream back to points; better, a `CutLine` newtype minted only by `patal-pattern` so the breach is unrepresentable |
| F3 | high | Nothing in the repo writes files; `pattern/src/lib.rs:325-327` defers I/O explicitly. §3.8 inherits nothing and mentions none. | `patal-export` returns bytes; the harness does temp-file + rename |
| F4 | high | `usable = page_w − 2·margin − overlap`. At `overlap >= usable` the tiling loop makes no progress — a hang, not an error. | Reject `usable <= 0` and `overlap >= usable`; cap tiles with `TooManyTiles` |
| F6 | high | Piece 7 of N failing after 6 are laid out yields a plausible-looking 6-piece PDF. That is the C1 violation that reaches cloth. | Compute every cut line before emitting a byte; fail whole-export naming the piece |
| F7 | medium | C12's 50mm square sits in page space and can land on the piece, making it a cut hazard. | Reserve a margin strip, dash it, label "50mm — do not cut", assert no intersection with any outline |
| F8 | high | Byte-stable PDF golden needs a pinned `/CreationDate`, no compression, exact xref offsets, deterministic float formatting. It goes red on an unrelated bump and gets re-blessed unread at 2am. | Two tiers: a semantic test on parsed content, plus the byte golden with a readable diff dump |
| F9 | high | Both proposed tests are per-page and pass while assembly is wrong. An off-by-one overlap duplicates or drops a strip. | Reassemble tiles from their own registration marks; assert the union equals the single-page outline |
| **F10** | high | **PDF strokes are centred on the path.** 1pt at 1:1 is ~0.35mm, comparable to the 0.4mm cutter tolerance at `curves.rs:18`; a `1 w` under an mm-scaled CTM is a 1mm line — ±0.5mm of ambiguity about where to cut. | Set line width in points explicitly (hairline, <=0.25pt); ADR-008 states the true line is the stroke centre |
| F11 | medium | `/MediaBox` correct is not the same as true scale: consumer printers have non-printable edges and some drivers still shrink under "Actual size". | `/MediaBox` = exact sheet; content inset to a stated printable margin; §3.9 measures on two printers |
| E-4 | medium | PDF library choice deferred to implementation; it sets the dependency tree and `cargo deny` surface. | Hand-roll (~350 LOC, zero deps) — `deny.toml` allows MIT/Apache-2.0 so licensing is not the constraint; determinism for F8 is |

**Plan correction.** §6 line 421 reasons `C2 no unsafe — PASS` from the export crate declaring
`#![forbid(unsafe_code)]`. That attribute binds only the crate declaring it, never its
dependencies. The conclusion may still hold; the stated reasoning does not.

**Test plan artifact:** `~/.gstack/projects/patal/satex25-main-test-plan-20260813-143000.md`
(test diagram, 13-row coverage matrix, 9 gaps, and the absolute-scale constant
1mm = 2.834645669 pt / 50mm = 141.7322835 pt).

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

Auto-decided by `/autoplan` using its 6 principles. P1 completeness, P2 boil lakes,
P3 pragmatic, P4 DRY, P5 explicit over clever, P6 bias toward action.

| # | Phase | Decision | Class | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | CEO | Mode = SELECTIVE EXPANSION | Mechanical | autoplan default | Iteration on an existing system | EXPANSION, HOLD, REDUCTION |
| 2 | CEO | Approach A (content) — keep D2's ordering | Mechanical | P1 | The informed-freeze argument is sound and independently corroborated by `roadmap.md` | B (export-first on v1) |
| 3 | CEO | Do not reduce scope despite 20+ files | Mechanical | P2 | Reduction is not an autoplan option; boundaries are the fix, not cuts | Cutting §3.1-§3.5 |
| 4 | CEO | E1 pre-registered friction-log questions → **accept** | Taste | P1 | Separates tool friction from novice friction; ADR-006's validity depends on it | Defer |
| 5 | CEO | E2 printer model + driver settings in provenance → **accept** | Mechanical | P1 | Zero cost, and §3.9's measurement is unfalsifiable without it | Skip |
| 6 | CEO | E3 multi-piece nesting → **defer to TODOS.md** | Mechanical | P3 | 2D bin packing with grain constraints is a separate problem, not a page transform | Accept now |
| 7 | CEO | E4 piece metadata on the page (name, grain, notches) → **accept** | Taste | P1+P2 | §3.10 predicts the sewer asks for exactly these; grain line lands in the same wave | Defer |
| 8 | CEO | E5 structural PDF assertion alongside the byte golden → **accept** | Mechanical | P1 | Independently reached by both review voices; byte-stability breaks on any bump | Skip |
| 9 | CEO | E6 flatten tolerance recorded in PDF provenance → **accept** | Mechanical | P1 | The artifact is the log for a cut-path tool | Skip |
| 10 | CEO | DXF conformance *verification* → **drop**; keep the capture | Taste | P3 | Capture costs minutes with Seamly2D already open; verification is the expensive half and DXF is deferred anyway | Cut DXF wholesale (subagent #7) |
| 11 | CEO | §3.4 primitive list drafted from domain knowledge first | Taste | P3 | Textbooks give the vocabulary; the tools answer the *persistence* question textbooks cannot | Delete §3.1-§3.5 (subagent #5) |
| 12 | CEO | §3.6 folded into §3.7 step 1 | Mechanical | P3 | A node whose expected output is "no drift" is a check, not a DAG node | Keep as its own node |
| 13 | CEO | Add vector-editor workflow as a named ADR-006 comparison axis | Taste | P1 | Illustrator + print shop is the realistic default for the target user | Add a third full draft |
| 14 | Eng | E-1 add `crates/export` to workspace members | Mechanical | P5 | Silent-failure gap; one line | Rely on the §6 claim |
| 15 | Eng | E-2/E-3 export calls `Project::cut_boundary(piece)`, passes no tolerance | Mechanical | P4+P5 | One tolerance in the file governs both the saved and printed cut line | Export owns a tolerance |
| 16 | Eng | F3 export returns `Vec<u8>`; harness owns file I/O | Mechanical | P5 | Keeps C3 clean, makes the golden test trivial | Atomic write inside export |
| 17 | Eng | F1 `CutLine` newtype minted only by `patal-pattern` | Taste | P5 | Makes the C11 breach unrepresentable instead of merely tested | Test-only enforcement |
| 18 | Eng | F4/F5/F6 named error variants + all-or-nothing emission | Mechanical | P1 | C1 is the project's own doctrine; a partial PDF still prints | Discover at runtime |
| 19 | Eng | F9 tile-reassembly test added | Mechanical | P1 | The only test that catches an assembly error | Per-page tests only |
| 20 | Eng | F10 stroke width pinned in points, hairline | Mechanical | P1 | ±0.5mm of cut ambiguity is a cut-path defect | Leave to implementation |
| 21 | Eng | E-4 hand-roll the PDF writer (~350 LOC) | Taste | P5 | Determinism for the golden test, zero new `deny.toml` surface | `printpdf` / `lopdf` |
| 22 | Eng | F11 §3.9 measures on two printers | Mechanical | P1 | One printer cannot distinguish a driver bug from a geometry bug | Single printer |

## DX findings (Phase 3.5) — audiences: the future maintainer of `patal-export`, and the operator printing a pattern

| # | Severity | Finding | Fix |
|---|---|---|---|
| X-1 | **critical** | **Paper-size mismatch.** An A4 PDF on a Letter tray scales ~94% silently. This is a *tray* setting, not "fit to page", so every §3.9 instruction can be followed correctly and the print is still 6% small. | `docs/setup/printing.md` states: driver paper size must equal the PDF page size. Verify before measuring. |
| X-2 | **critical** | Units are the entire risk and are unnamed. `cut_boundary()` is mm; PDF user space is 1/72in. The single multiply that converts them is where a wrong physical pattern is born. | Newtypes `Mm(f64)` / `Pt(f64)` and one `const MM_PER_PT`, so a raw `f64` cannot reach the page transform |
| X-3 | high | **No tolerance number exists in the plan.** S6 says "within tolerance"; §5.5 says "declared in advance"; nothing declares it. | State it before §3.9 runs. Proposed: ±0.5mm over 200mm |
| X-4 | high | The 50mm square is too small to be the instrument: at 1% error it is off 0.5mm, inside ruler noise. Feed-direction and cross-direction printer scale also differ. | Add a 200mm ruled line on **both** axes alongside the square |
| X-5 | high | No API is specified anywhere — no type, no function, no entry point. A maintainer in six months has no contract. | Put the signature in the plan as an acceptance criterion: `pub fn export_tiled_pdf(pieces: &[&PatternPiece], layout: &PageLayout) -> Result<Vec<u8>, ExportError>` with `PageLayout::a4()`/`::letter()` |
| X-6 | high | §3.8 inputs say "a `PatternPiece`" but a bodice block is 3-5 pieces. Single-piece means the operator prints 5 tile grids and hand-collates. | Take a slice; state the v1 placement rule (one piece per grid origin, no nesting) |
| X-7 | high | Nothing forbids a `scale` parameter, and the first person wanting a half-size check will add one. | No scale parameter. If ever added it stamps `NOT TO SCALE` on every page and in the filename |
| X-8 | high | `margin_mm` below the printer's hardware unprintable area (~4-6mm on inkjets) silently clips the cut line. | Default 10mm; warn below 6mm |
| X-9 | medium | `docs/setup/` holds only `toolchain.md` and `reference-repositories.md`. §3.9's runbook gets one bullet and no filename. | Create `docs/setup/printing.md` with per-reader dialog wording |
| X-10 | medium | Tile assembly convention is unstated: trim to a line, or overlay marks? Double-counting overlap grows the piece by `overlap x (rows-1)`. | State the convention in ADR-008 and print it on the page |
| X-11 | medium | Golden-test instability also comes from the PDF `/ID` field, not only `/CreationDate`. | Pin both to constants |

**DX scorecard.** API ergonomics 3/10, error messages 2/10, printing runbook 3/10,
time-to-first-correct-print 4/10, defaults and escape hatches 4/10. **Overall 3.2/10 → target 8/10.**

## Cross-phase themes

Concerns that surfaced independently in more than one phase. These are the high-confidence signals.

1. **The byte-stable golden PDF test will churn and then be blessed unread.** Flagged in CEO,
   Eng (F8) and DX (X-11) independently. Three of three phases.
2. **The C11 assertion cannot catch a scale error.** CEO, Eng (F1), and the test plan (T3).
   C11 is still the right rule; the test that enforces it is blind in the one direction §3.9
   calls most likely.
3. **The plan sets falsifiable-looking checks and omits the number.** ADR-006's "no wedge" verdict
   gates nothing (CEO #2); S6's tolerance is never declared (DX X-3); "validated ranges" names no
   ranges (Eng, DX). Same failure shape in three places.
4. **Page-parameter validation is missing and one case is a hang, not an error.**
   `overlap >= usable` makes the tiling loop stop progressing. Eng (F4) and DX (§5).
5. **The DAG edge §3.7 → §3.8 is one function signature wide.** CEO voice #1 and eng E-2 reached
   this from opposite directions; the architecture diagram above shows it structurally.

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 1 | issues_open | mode: SELECTIVE_EXPANSION, 4 critical gaps, 13 decisions |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | CLI not installed |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | issues_open | 13 issues, 4 critical gaps |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | skipped | no UI scope detected |
| DX Review | `/plan-devex-review` | Developer experience gaps | 1 | issues_open | score 3.2/10 → 8/10 target, 11 issues |

- **CROSS-MODEL:** not available. Codex CLI is not installed, so all three phases ran
  `[subagent-only]`. Every consensus row is one independent voice plus the primary review, never
  two models. Install `@openai/codex` for genuine cross-model coverage.
- **VERDICT:** CEO + ENG + DX reviewed, all three with issues open. 34 findings, 8 critical.
  Not ready to implement as written — the DAG re-cut and the four cut-path gaps land first.

**UNRESOLVED DECISIONS:**
- Whether to re-cut the DAG so export + print + measure no longer waits on the v2 freeze
- Whether ADR-006's "no wedge" verdict gets pre-committed halt criteria, or stays advisory
