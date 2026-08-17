---
title: Incumbent persistence probe — what Seamly2D and Freesewing actually store
date: 2026-08-16
status: evidence — format-level only, K3 friction not yet run
tags: [analysis, domain, schema, incumbents, evidence]
---

# Incumbent persistence probe

> Answers the *persist or draw* question from
> [the pattern primitive census](pattern-primitives.md) for the two rows that gate the
> v2 schema freeze — **P-09 (dart)** and **P-21 (material per cut instance)**, with
> **P-17** and **P-20** riding on the second. It does **not** answer the census's other
> rows, and it is **not** ADR-006. See [limits](#what-this-cannot-tell-you).

The census closed with three decisions standing between the project and the v2 freeze.
Decision 1 was taken on 2026-08-15. Decisions 2 and 3 were left open **on evidence** —
the census was explicit that resolving them on taste was the failure mode to avoid. This
document supplies the evidence obtainable without a running copy of either tool, and says
plainly where that stops.

**Headline, before the citations.**

- **A dart is not an object in either incumbent.** Seamly2D persists a dart as three
  point references consumed by a truing *tool*; Freesewing's core and every one of its
  plugin source trees contain the string `dart` **zero times**. Darts exist there only
  inside individual designs, as ordinary paths.
- **Seamly2D built material-on-the-cut, shipped it for eighteen schema versions, and
  then deleted it** — migrating structured cut instructions into label prose, and
  destroying the per-cut quantity in the process. This is the most decision-relevant
  fact in either tree and it is a warning, not a precedent.
- **Freesewing has material-on-the-cut today**, in close to the shape P-21 predicted.
  Its version of the structure is untyped, and the corpus contains a silent
  wrong-material bug as a direct result.

---

## Method, and where it departs from the census protocol

The census specified two protocols. One was run as written; the other was substituted,
and the substitution is the first thing a reader should be able to audit.

**Freesewing — run as written.** The census said: *"Search the core package's public API
and its documentation for the construct. A named macro, snippet or method means the
system models it; nothing means each designer draws it by hand."* That is a source read,
and it was executed exactly. No deviation.

**Seamly2D — substituted.** The census specified a save/add-one-construct/save/diff
protocol, which needs the application, which needs K3's drafted block. That has not
happened. Instead this probe reads **the format definitions** — Seamly2D ships 49
versioned XSDs for its pattern file — plus the C++ that writes, reads and *migrates*
them.

The substitution is stronger in one direction and weaker in the other, and both matter:

- **Stronger than the diff on coverage, and on history.** An XSD enumerates every
  element and attribute the format can hold, by name, with type. The diff protocol
  samples the format one construct at a time and finds only what the operator thought to
  draw. Reading *all 49 versions plus the converter* additionally exposes what the format
  **used to** hold and why it stopped — which is where this probe's most important
  finding came from, and which no diff of the current version could ever have surfaced.
- **Weaker than the diff on behaviour.** The schema says what the format *can* express.
  It cannot say what the application writes for a given user action, whether a nominally
  optional attribute is written in practice, or whether a construct is reachable from the
  UI at all. Where this bites, it is flagged.

Both readings answer *format-level persistence*, which is what a schema freeze turns on.
Neither answers *whether the tool is pleasant to draft in*, which is what ADR-006 turns
on. Different questions; this document claims only the first.

## Sources

| Tool | Ref | Read | What was read |
|---|---|---|---|
| Seamly2D | `github.com/FashionFreedom/Seamly2D` @ `d6e7562` (2026-08-09, default branch) | 2026-08-16 | all 49 files in `src/libs/ifc/schema/pattern/` (`v0.1.0` – `v0.7.4`; the newest is 1050 lines), `src/libs/ifc/xml/vpatternconverter.cpp`, plus the tool and label sources named per finding |
| Freesewing | `github.com/freesewing/freesewing` @ `8a8de5a` (2025-04-02, `develop`, the remote's default branch), core `v4.0.0` | 2026-08-16 | `packages/core/src/`, all `plugins/*/src/`, and `designs/*/src/` for the absence checks and the corpus counts |

Freesewing's `develop` head being sixteen months old is recorded because it is
load-bearing for one claim below, not as a remark about the project: the "zero dart
support" finding is a statement about `8a8de5a`.

The current Seamly2D native extension is `sm2d` (`src/libs/vmisc/def.cpp:193`); `val`
(`:190`) is the legacy Valentina extension, still readable. "The pattern file" below means
whichever the version in question wrote.

---

## P-09 · Is a dart an object?

The census asked two things: *"Is a dart an object with an apex and an intake, or is it
two internal lines plus a manually-trued outline? Does the file record the dart such that
closing it recomputes the boundary, or is truing the designer's problem?"*

### Seamly2D — three point ids and a truing tool. Not an object.

There is no `<dart>` element in any of the 49 pattern schemas. The word appears in the
newest one exactly three times, as attributes on the generic `<point>` element:

```xml
<!-- src/libs/ifc/schema/pattern/v0.7.4.xsd:194-196 -->
<xs:attribute name="dartP1" type="xs:unsignedInt"/>
<xs:attribute name="dartP2" type="xs:unsignedInt"/>
<xs:attribute name="dartP3" type="xs:unsignedInt"/>
```

`xs:unsignedInt` is Seamly2D's object-id type throughout the schema, so these are
*references to three points the designer already drew* — not coordinates, and not a
shape. They belong to one tool:

```cpp
// src/libs/vtools/tools/drawTools/toolpoint/tooldoublepoint/vtooltruedarts.cpp:76
const QString VToolTrueDarts::ToolType = QStringLiteral("trueDarts");
```

What that tool does is the answer. It takes a base line and the three dart points and
emits **two points**:

```cpp
// src/libs/vtools/tools/drawTools/toolpoint/tooldoublepoint/vtooltruedarts.cpp:102-104
void VToolTrueDarts::FindPoint(const QPointF &baseLineP1, const QPointF &baseLineP2, const QPointF &dartP1,
                               const QPointF &dartP2, const QPointF &dartP3, QPointF &p1, QPointF &p2)
```

So: the dart's *legs* are ordinary lines the designer drew. The dart's *truing* is a tool
that rotates the base line through the dart's angle and intersects it, producing the two
boundary points a folded-and-cut edge would leave. Those two points are then ordinary
points, which the designer must remember to include in the piece's path.

**Both halves of the census's question, answered.** A dart is not an object with an apex
and an intake — it is two lines plus a named truing operation. And closing the dart does
not recompute the boundary: it computes two points, and putting them on the boundary is
the designer's problem.

There is a third thing here the census did not ask for, and it is the more interesting
finding. What the file stores for a `trueDarts` point is **only the five ids**:

```cpp
// vtooltruedarts.cpp:416-424 — SaveOptions writes ids, never coordinates
doc->SetAttribute(tag, AttrBaseLineP1, baseLineP1Id);
doc->SetAttribute(tag, AttrBaseLineP2, baseLineP2Id);
doc->SetAttribute(tag, AttrDartP1, dartP1Id);
doc->SetAttribute(tag, AttrDartP2, dartP2Id);
doc->SetAttribute(tag, AttrDartP3, dartP3Id);
```

The trued points are therefore **derived on every parse**, not baked at authoring time,
which is what puts them in the dependency graph and makes them follow a measurement
change. **One qualifier, and it is a real one:** an edit to an external measurement file
does not push. It sets a dirty flag through a `QFileSystemWatcher`, and the re-parse runs
in `syncMeasurements()` → `LiteParseTree()`, reached from the main window's **focus-in**
handler (`src/app/seamly2d/mainwindow.cpp:306`). The propagation is real; it is
pull-on-focus, not push-on-write.

Seamly2D gets the propagation property without ever modelling a dart, purely by making
the truing step a node in the graph. That is a genuinely different answer from either
option the census posed — see [what this means for Decision 2](#decision-2).

### Freesewing — nothing in core, everything in designs. Drawn by hand.

Searched `packages/core/src/` and every `plugins/*/src/` tree, case-insensitive, over
`.mjs`:

```
$ grep -rEoi --include=*.mjs "dart" packages/core/src plugins/*/src | wc -l
0
```

By the census's own evidence standard that is a finding, not a failure to find. The
plugin that would hold it if anything did is `plugin-annotations`, which ships eleven
macro groups (`plugins/plugin-annotations/src/index.mjs:50-62`) across sixteen source
files — `bartack`, `cutlist`, `cutonfold`, `dimensions`, `grainline`, `notches`, `pleat`,
`scalebox`, `sewtogether`, `title` and others. None is a dart.

Where darts do appear is inside individual designs, as ordinary path geometry:

```
$ grep -rEoi --include=*.mjs "\bdarts?\b" designs/*/src/ | wc -l
211
```

```js
// designs/simone/src/fba-front.mjs:319
paths.dart = new Path().move(points.dartBottom).line(points.bustDartTip).line(points.dartTop)
```

And the consequences of having no dart object are visible in how designs work around it.
`designs/charlie` reserves a named slot in a path and splices dart geometry into it
later:

```js
// designs/charlie/src/back.mjs:38   — reserve a slot between backDartRight and backDartLeft
      .noop('dart')
// designs/charlie/src/back.mjs:151  — splice geometry into that slot
    .insop('dart', new Path().line(points.pocketCenter))
```

`noop`/`insop` is Freesewing's generic path-splicing mechanism, and here it is being used
so that `paths.seam` includes the dart V while `paths.saBase` — the path the seam
allowance is offset from — omits it. **That is dart-exclusion from the allowance base, not
trueing**, and it is worth being precise about: it does not show a designer hand-trueing a
waistline. What it does show is that the dart's interaction with the boundary has to be
hand-wired by every design that has a dart, because nothing in the system knows that a
dart is a dart.

**Verdict for P-09, persistence half: neither incumbent models a dart as an object.** One
derives the trued points through its dependency graph; the other leaves the whole thing to
the pattern author.

---

## P-21 · Does material belong to the piece or to the cut?

### Seamly2D — it did. Then it deleted it, lossily.

This is the finding the probe exists to produce, and it inverts the argument the evidence
first appeared to make.

From `v0.3.3` through `v0.5.1` — **eighteen consecutive schema versions** — the piece's
label block carried an unbounded list of `<mcp>` children. Material, Cut number,
Placement:

```xml
<!-- src/libs/ifc/schema/pattern/v0.5.1.xsd:495-501 -->
<xs:element name="mcp" minOccurs="0" maxOccurs="unbounded">
  <xs:complexType>
    <xs:attribute name="cutNumber" type="xs:unsignedInt"/>
    <xs:attribute name="userDef"   type="xs:string"/>
    <xs:attribute name="material"  type="materialType"/>
    <xs:attribute name="placement" type="placementType"/>
  </xs:complexType>
</xs:element>
```

`materialType` (`v0.5.1.xsd:842-855`) is an enum — `0=Fabric, 1=Lining, 2=Interfacing,
3=Interlining, 4=UserDefined` — and `placementType` is `0=No placement, 1=Cut on Fold`.
So: **a list of
cut instructions per piece, each carrying its own material, its own quantity, and its own
cut-on-fold flag.** That is `Vec<CutInstruction>`, shipped, in 2016-era XML.

**It was removed in v0.6.0, on purpose, and migrated into label text.** The converter
rewrites each `<mcp>` into a line of prose:

```cpp
// src/libs/ifc/xml/vpatternconverter.cpp:2862-2883 — inside PortPieceLabelstoV0_6_0() (:2832)
switch(material)
{
    case 0:  line.append("%mFabric%");       break;
    case 1:  line.append("%mLining%");       break;
    case 2:  line.append("%mInterfacing%");  break;
    case 3:  line.append("%mInterlining%");  break;
    case 4:
    default: line.append(GetParametrString(domMCP, strUserDefined, "User material")); break;
}

line.append(", %wCut% %pQuantity%");
```

and then deletes every `<mcp>` node (`RemoveUnusedTagsV0_6_0()`, `:2895`, removal loop at
`:2906-2915`).

**The migration is silently lossy, and the loss is exactly the field P-21 is about.**
`%pQuantity%` resolves to the piece's *single* `quantity` attribute
(`src/libs/vlayout/vtextmanager.cpp:153` — `placeholders[pl_pQuantity] =
QString::number(data.GetQuantity())`). Each `<mcp>` had its **own** `cutNumber`. The
converter never reads it:

```
$ grep -n "strCutNumber\|cutNumber" src/libs/ifc/xml/vpatternconverter.cpp
(no matches — the attribute is read nowhere in the current source; it survives only in
 the eighteen old XSDs that declare it)
```

A v0.5.1 piece recorded as *cut 2 in fabric, cut 1 in interfacing* therefore migrates to
two label lines that both print the same number. The document still looks right on the
page. It is no longer right in the file, and nothing tells anyone.

For a project whose kernel header reads *"every operation here is either correct or loud
— a pattern piece that is silently wrong is worse than one that refuses to compute"*, this
is the sharpest possible cautionary example, and it lands on machinery Pātāl is building
right now: **the SeamPath blueprint's §3.7 migration step.** A migration that cannot
represent the old data must fail loudly, not round it off. Seamly2D's rounds it off.

**What is left today** is the residue of that migration. Quantity survives, as an
attribute of the piece's *label*:

```xml
<!-- src/libs/ifc/schema/pattern/v0.7.4.xsd:566 — an attribute of pattern/pieces/piece/data -->
<xs:attribute name="quantity" type="xs:unsignedInt"/>
```

`<data>` is the label block; `quantity` sits among `letter`, `annotation`, `orientation`,
`foldPosition` and `onFold` — all attributes of the same element, all about what gets
printed. Material survives only as **label template placeholders**:

```cpp
// src/libs/vmisc/def.cpp:140-144
const QString pl_mFabric       = QStringLiteral("mFabric");
const QString pl_mLining       = QStringLiteral("mLining");
const QString pl_mInterfacing  = QStringLiteral("mInterfacing");
const QString pl_mInterlining  = QStringLiteral("mInterlining");
```

which resolve, at render time, to translated **words**:

```cpp
// src/libs/vlayout/vtextmanager.cpp:134
placeholders.insert(pl_mFabric, QObject::tr("Fabric"));
```

`%mFabric%` is a localisation token for the string "Fabric". It carries no identity, no
properties, and nothing can be looked up through it. Handedness is elsewhere again, on the
piece, and expressed as a *prohibition* rather than an instruction —
`forbidFlipping` (`v0.7.4.xsd:668`) tells the nesting engine it may not mirror this piece
when packing a marker. It cannot say "cut two, mirrored"; the label says that, in prose.

**So today Seamly2D binds material to neither the piece nor the cut. But it is not a tool
that never had the idea — it is a tool that had it and traded it away.**

### Freesewing — on the cut, in close to the shape P-21 predicted

`plugin-annotations` ships a `cutlist` **store API** (not a macro — `cutlistStores`,
`index.mjs:63`). Its documented signature:

```js
// plugins/plugin-annotations/src/cutlist.mjs:30-34
 * @param {number} so.cut = 2             the number of pieces to cut from the specified fabric
 * @param {string} so.from = fabric       the name of the material to cut from
 * @param {boolean} so.identical = false  should even numbers of pieces be cut in the same direction or mirrored
 * @param {boolean} so.onBias = false     should the pieces in these cutting instructions be cut on the bias
 * @param {boolean} so.onFold = false     should these cutting instructions ignore any cutOnFold information set by the part
```

And the storage, which is the part that matters:

```js
// plugins/plugin-annotations/src/cutlist.mjs:57-59
const path = ['cutlist', partName, 'materials', from]
const existing = store.get(path, [])
store.set(path, existing.concat({ cut, identical, onBias, onFold }))
```

`concat`, not `set`. A part holds a **list** of cut instructions **per material**. P-21
predicted that quantity, material, handedness and role would all attach to the cut and
that the entity would be a `Vec<CutInstruction>`; `cut`, `from` and `identical` are the
first three of those, and the prediction is confirmed rather than extended by them.

Two attributes P-21 did **not** anticipate, and their scope is narrower than it looks:

- **`onBias`** — cut on the bias.
- **`onFold`** — an *override* of the part's own cut-on-fold setting.

**Both are booleans on the cut, but the geometry they refer to is not.** The grain angle
is stored once per part (`cutlist.mjs:78` — `store.set(['cutlist', partName, 'grain'],
grain)`), and the cut-on-fold **points** are stored once per part (`:110`). So a part
cannot actually carry a different grain angle or a different fold edge per material. The
booleans change a printed note, not a geometric fact. That is a weaker claim than "grain
is per-cut", and it is the claim the source supports.

The cut list is also where other macros deposit structured facts. `grainline` and
`cutonfold` each draw their annotation *and* record:

```js
// plugins/plugin-annotations/src/grainline.mjs:75
store.cutlist.setGrain(mc.from.angle(mc.to), 'grainline')
// plugins/plugin-annotations/src/cutonfold.mjs:81-82
store.cutlist.setCutOnFold(mc.from, mc.to)
if (mc.grainline) store.cutlist.setGrain(mc.from.angle(mc.to), 'cutonfold')
```

**The two-layer split is the design idea worth taking, and Freesewing does not yet cash it
in.** The macro emits the mark a human reads and separately writes the fact a machine
could read. But `identical`, `onBias` and `onFold` have exactly one consumer in the whole
repository — `title.mjs:166-176`, which turns them back into printed note strings. The
only other structured reader of the cut list is `getCutFabrics`, which returns material
names. So the machine layer currently exists to generate the human layer. The
*architecture* separates them; the *system* has not yet found a second use for the
separation.

That is still the right lesson for Pātāl, and it is worth stating precisely: Pātāl today
has only the drawing layer — `patal-export` draws — and nothing that would let a lay plan,
a cutting ticket or a DXF layer inventory ask *"what does this piece need cut, from what,
how many, mirrored or not"*. TODOS' P2 nesting item is a consumer of exactly that layer,
and it does not exist to consume.

### P-20 · Role — and what an untyped material slot costs

Neither tool has a role attribute today. Seamly2D's `materialType` enum **was** one —
`Fabric / Lining / Interfacing / Interlining / UserDefined` is a role taxonomy with an
escape hatch — and it went out with `<mcp>`. Freesewing folds role into the material name.
The `from` values across its design corpus:

| `from` value | uses |
|---|---|
| `'fabric'` | 207 |
| `'lining'` | 36 |
| `'interfacing'` | 18 |
| `'ribbing'` | 12 |
| `'Fabric'` | 12 |
| `'canvas'` | 9 |
| `'undersideFabric'` | 2 |
| `'special'` | 1 |
| `'rigidInterfacing'` | 1 |
| `'altFabric1'` | 1 |

`fabric` / `lining` / `interfacing` are **roles**; `ribbing` / `canvas` are **materials**;
`undersideFabric` and `rigidInterfacing` are both at once. The taxonomy has no axis
because the field has no type.

`'Fabric'` beside `'fabric'` looks like the punchline and **is not**: all twelve uses are
in `designs/skully`, which never uses the lowercase spelling, and the cut list is
per-pattern — so no pattern asks for two fabrics. The real cost of the untyped slot is
elsewhere and is worse, because it is silent:

```js
// designs/bee/src/cup.mjs:220
store.cutlist.addCut({ cut: 2, material: 'altFabric1', ignoreOnFold: true })
```

The option keys are `from` and `onFold`. `material` and `ignoreOnFold` are not options
`addCut` knows. It validates only `typeof from !== 'string'` (`cutlist.mjs:53`), so both
unknown keys are ignored and `from` falls back to its default. **These two pieces are
recorded as cut from `fabric` when the design says `altFabric1`.** No warning, no log
line — `addCut` has a `store.log.warn` path and this does not reach it. A piece cut from
the wrong cloth, in the shipped corpus, produced by an options bag that accepts any shape.

This is [ADR-004](../adr/ADR-004-document-format.md)'s argument arriving from outside, and
`patal-pattern`'s `MaterialNotFound` doc comment already states the principle it violates:
*"a piece that silently forgets its material is a piece that gets cut from the wrong
cloth, and the person who finds out is the person holding the scissors."* Pātāl's typed
`MaterialId` makes `bee/cup.mjs:220` a compile error rather than a defect. **That belongs
in ADR-006 when it is written — as an observation with a citation, not a feature
comparison.**

---

## What this settles, and what it does not

### Decision 2

> *Is a dart an object? If yes, the authored outline stops being authoritative and
> `cut_boundary()` becomes a function of outline and darts.*

**Not decided here, and deliberately.** The census said "Do not decide this before K3",
and K3 — the friction of drafting a bodice block — has not run. That instruction stands.

What has changed is the shape of the remaining question. Before this probe, one plausible
reading was that Pātāl was behind on a construct both incumbents had solved. That reading
is dead: **neither incumbent has a dart object.** The choice is not "catch up or diverge".
It is:

1. **Dart as object** — nobody ships this. A genuine wedge if it works, and a genuine
   research risk if it does not, because there is no prior art and no evidence anyone has
   found it worth building.
2. **Seamly2D's answer: dart as a derived operation in the dependency graph.** The tool
   stores five ids and re-derives the trued points on parse, so they follow a measurement
   change without a dart type existing. This is the propagation the memorandum demands, at
   a fraction of the schema cost, and it lands squarely on the constraint/propagation
   solver that `docs/roadmap.md` already names as the biggest unbuilt pillar. **If the
   solver is coming anyway, option 2 may be nearly free.**
3. **Freesewing's answer: the designer's problem.** Rejected on the memorandum's own terms
   — it is the "drawing tool with a garment theme" the project exists not to be.

Option 2 was not on the census's list, and it is the one K3 should now be run to
discriminate against option 1. That is a sharper question than K3 was going to answer
before, which is the value this probe adds to a decision it does not make.

### Decision 3

> *Does material belong to the piece or to the cut?*

**The persistence half is answered — to the cut — and the answer arrives with a warning
attached.**

Both incumbents converge on the *shape*: a per-piece list of cut instructions, each
carrying material, quantity and a fold flag. Freesewing has it now; Seamly2D had it for
eighteen schema versions. The census predicted that shape from domain reasoning alone,
before either tool was opened, and two independent implementations match it. **That is as
much confirmation as a schema decision can get without building it.**

The warning is that one of the two **withdrew**. Seamly2D moved structured cut
instructions into label prose in v0.6.0 and let the per-cut quantity fall on the floor.
This probe cannot say *why* — the rationale is not in the code, and finding it means
reading issue history that has not been read. Until someone does, the honest reading is:
**a mature team decided that flexible label templates were worth more than structured cut
data, and Pātāl should know what they knew before repeating or rejecting the trade.** That
question is now the highest-value unresolved item in this whole area, and it is
answerable — it is a history read, not a drafting session.

Two consequences that are decision-relevant and **not** decided here:

- **A `CutInstruction` must carry a `MaterialId`, not a name.** `bee/cup.mjs:220` is what
  the alternative costs, and ADR-004 already bought the fix.
- **Quantity must live on the cut, not beside it.** Seamly2D's surviving
  piece-label `quantity` is precisely the degraded form left behind when per-cut quantity
  was dropped, and it is the shape Pātāl would land in by accident if it added a quantity
  field to `PatternPiece` rather than to a cut.

**Recommended framing for the freeze, not a decision:** `PatternPiece.material` is demoted
from a field to a default, `Vec<CutInstruction>` appears, and `CutInstruction` carries at
minimum `{ material: MaterialId, quantity, mirrored }`. `onBias` and `onFold` are
candidates whose only evidence is Freesewing's API surface — and whose geometry, in
Freesewing, is stored per part rather than per cut — so adding them now would be the
"fields nobody validates" failure the census warns about at P-10. **The operator owns
whether this lands at v2 or v3.** It is structural either way, which is why it is written
down rather than implemented.

## Scoring the pre-registered guesses

Only guesses this probe touched. The rest stand unscored.

| # | Guess | Outcome |
|---|---|---|
| G1 | Seamly2D's project file is XML and inspectable | **Survives**, and stronger than guessed — 49 versioned XSDs plus a converter, which is why the diff protocol could be substituted rather than deferred, and why the `<mcp>` history was visible at all |
| G3 | Seamly2D persists notches and grain lines as named entities | **Survives.** Notches are node attributes — `notch` (`v0.7.4.xsd:389`), `notchType` (`:390`), `notchSubtype` (`:391`), `notchLength` (`:395`), `notchWidth` (`:396`), `notchCount` (`:397`) — and grainline is an element with `arrows`, `length`, `rotation`, `arrowLength` |
| G5 | Freesewing's core exposes named helpers for title, grain line, cut-on-fold and notches | **Survives with one correction.** `title`, `grainline` and `cutonfold` are macros; **`notches` is not** — it ships defs only (`notches.mjs:2`, `export const notchesDefs`) and is consumed as `new Snippet('notch', …)`. The construct is named and modelled; the mechanism is a snippet, not a macro |
| G6 | Neither tool models seam pairing as first-class data — *"the guess most worth being wrong about"* | **Survives narrowly on the Freesewing side, and is under-tested.** `sewtogether` is a pure annotation macro: it takes two **Points** within a single part (`sewtogether.mjs:68-80`), never a second part and never an edge, and writes back only its own generated node ids (`:121`). But only that one file was examined — the protocol was not run across the core API for seam pairing, and the Seamly2D side was not tested at all |

Three of the four survived cleanly, which by the census's own rule 4 is *"worth suspecting
rather than celebrating"*. The honest reading: G1, G3 and G5 were low-risk guesses about
whether a mature tool has features, and a format-level read is the shallowest kind of
contact. G6 is the one that carried risk and it is barely tested.

**One prior that was not pre-registered, and was wrong.** This probe's own first draft
concluded that Seamly2D "does not model material as data at all" from the current schema
alone. Reading the older schemas and the converter reversed it. Recorded because the
census's discipline is that being wrong should be visible: **a format-level read of the
newest version is not the same as a read of the format.**

## What this cannot tell you

Recorded so a later cycle does not mistake this document for more than it is.

1. **It is not K3.** No block has been drafted in either tool, no friction log exists, and
   [ADR-006 must not be written from this document](pattern-primitives.md#filling-this-in)
   — it is a census with citations, exactly the artifact the census's rule 5 forbids as a
   source for the wedge.
2. **Schema coverage is not application behaviour.** Everything claimed about Seamly2D is
   a claim about what the format holds and what the source does with it. Whether the
   application writes a given attribute in practice, and whether a construct is reachable
   from the UI, is still a diff-protocol question.
3. **The `<mcp>` removal rationale was not found.** The migration code is unambiguous;
   the reason is not in it. Issue trackers, release notes and mailing lists were not read.
   **This is the highest-value follow-up in this document.**
4. **The Freesewing tree is sixteen months old** at the commit read. "Zero dart support"
   is a claim about `8a8de5a`.
5. **The census's other rows are untouched.** P-03, P-05, P-08, P-13 and the rest have
   corroborating evidence in the same two trees — the per-node `before`/`after` seam
   allowance attributes at `v0.7.4.xsd:386-387` are visible in material already read — and
   none of it has been collected row by row.
6. **No K6 verdict column was added** to the census. K6 is defined as the output of contact
   with both tools; adding verdicts from half of that contact would overwrite a
   pre-registration with a partial result, which is the one thing the census's rule 1
   forbids.

## Next

- **Find out why `<mcp>` was removed.** Highest value, lowest cost, and it is the
  difference between Decision 3 having a confirming precedent and a cautionary one.
- **Run K3.** It now has a sharper question than it started with: discriminate Decision 2
  option 1 (dart as object) against option 2 (dart as a derived operation in the
  dependency graph), rather than deciding whether a dart is an object at all.
- **Collect the remaining rows from the same two sources.** The material is cloned and the
  method is proven; the cost is a row-by-row read, not new access.
- **Test G6 against Seamly2D**, and run Freesewing's own protocol across the core API for
  seam pairing rather than one file. It is the guess most worth being wrong about and is
  currently barely scored.
