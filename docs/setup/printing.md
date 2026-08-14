# Printing a pattern at true scale

A pattern is only a pattern if a millimetre in the file is a millimetre on the
paper. Everything Pātāl does upstream of this page — the offset kernel, the
seam allowance validation, the typed errors — is worth nothing if the printer
quietly shrinks the result by six per cent, and it will, by default, given the
chance.

This page is the runbook for not letting it. Read it once before your first
print. After that the only step that matters every time is: **measure the
square**.

---

## The two-minute version

1. Print **page 1 only** — the calibration page.
2. Set the driver's **paper size** to the same size the PDF says it is.
3. Set scaling to **100% / Actual size**. Not "fit to page", not "shrink
   oversized pages".
4. Measure the two rules on the printed page from their shared corner.
5. If both read what they say they read, print the rest. If not, stop and read
   the troubleshooting section — do not adjust the scale until it looks right.

---

## The trap that catches everyone

**"Actual size" does not override the tray's paper size.**

If the PDF is A4 (210 × 297 mm) and the printer driver is set to Letter
(216 × 279 mm), most drivers scale the document to fit the sheet they think
they have. That is roughly 94%. Every scaling control in the print dialog can
be set correctly and the print is still six per cent small — a 100 mm bodice
width comes out 94 mm, and nothing on screen or on paper announces it.

This is a *tray* setting, not a *scaling* setting, which is why following the
scaling instructions carefully does not save you.

So: check the page size the PDF declares, and set the driver to match it.

- The calibration page prints its own page size in the provenance block.
- The harness's export report returns `page_size` for the same reason.
- `PageLayout::a4()` is 210 × 297 mm; `PageLayout::letter()` is 215.9 × 279.4 mm.

If you only have Letter paper, export with `PageLayout::letter()` rather than
printing an A4 document onto it.

---

## What is on the pages

**Page 1, the calibration page.** Two ruled lines meeting at a corner, one
across the page and one down it, each labelled with its true length. Plus a
50 mm square and the export's provenance: page size, margin, overlap, and the
sheet count and seam allowance of every piece.

Both axes are there because printers do not scale equally in both directions.
The paper-feed direction is driven by rollers and the carriage direction by a
belt, and a mechanical fault in one shows up in one axis only. A single rule
cannot tell a driver problem from a printer problem.

The lines are long on purpose. A 1% error moves a 50 mm mark by half a
millimetre, which is inside the noise of reading a steel rule. Over 200 mm the
same error moves the mark by two millimetres, which is not. The square is the
per-sheet check; the long rules are the instrument.

**Every other page** carries a 50 mm calibration square in a reserved strip at
the foot, plus the piece name, the sheet number and its grid position, and the
assembly note. The strip is reserved rather than overlaid: no cut line is ever
drawn across the square, because a box labelled "measure me" sitting on a line
labelled "cut me" is a genuine hazard.

On the pattern itself:

- **Solid black** is the cut line. This is the one the scissors follow.
- **Dashed grey** is the sewing line — the authored outline, one seam
  allowance inside the cut line.
- **Crosses** labelled `x1y0` and so on are registration marks.

Lines are drawn at 0.25 pt, about 0.09 mm. That is well inside the 0.4 mm the
geometry kernel treats as cutter tolerance, so which side of the printed line
you cut cannot matter. The true line is the centre of the stroke.

---

## Assembling the sheets

**Overlap the crosses. Do not trim.**

Sheets within a piece overlap by 10 mm by default. Each sheet is printed with
registration crosses on the shared model grid, and the same cross carries the
same label on both sheets that show it. Lay the next sheet over the previous
one so that `x1y0` sits exactly on top of `x1y0`, and tape.

Do not trim to the dashed window frame and butt the sheets together. Trimming
plus overlapping is the classic double-count: it grows the piece by
`overlap × (sheets − 1)`, which on a four-sheet piece is 30 mm of error that
looks like a slightly generous pattern rather than a mistake.

Sheets come out of the printer in assembly order: top row left-to-right, then
the next row down. Each sheet says which it is.

---

## Per-print checklist

- [ ] Driver paper size equals the PDF page size.
- [ ] Scaling is 100% / Actual size. No fit-to-page, no auto-rotate-and-centre
      if it offers to scale.
- [ ] Duplex off. A duplex path stretches paper.
- [ ] Print page 1, measure both rules **and** the square before printing the
      rest.
- [ ] Same paper tray and same paper for the whole job. Different stock feeds
      differently.

Record the printer model and the driver settings alongside any measurement you
write down. A measurement without them cannot be reproduced, and a
true-scale claim you cannot reproduce is a claim you do not have.

---

## When the numbers are wrong

Read this before touching a scale setting. Compensating with a scale factor
turns a diagnosable fault into a permanent one.

| What you measure | What it means | What to do |
|---|---|---|
| Both axes short by the same percentage | Fit-to-page, or a paper-size mismatch | Fix the driver's paper size first, then scaling |
| Both axes short by about 6% | An A4 document on a Letter tray, or the reverse | Match the page size; do not "correct" it with 106% |
| One axis right, the other wrong | Printer mechanics, not the document | Try another printer; the PDF is fine |
| Both axes right, square wrong | Not possible — re-measure the square centre-of-stroke to centre-of-stroke | — |
| Everything right, tiles do not meet | Assembly, not scale | You trimmed and butted; overlap the crosses instead |
| The square is right on some sheets, wrong on others | Mixed paper, or a driver scaling only some pages | Reprint the whole job in one pass |

If the printed geometry is wrong and the calibration marks are right, the bug
is in Pātāl, not the printer. That is worth saying plainly, because it is the
outcome this whole apparatus exists to be able to detect.

---

## Recording a measurement

The tolerance for this project is **±0.5 mm over 200 mm** — declared before any
measuring happened, on purpose, so that the number cannot be chosen to fit the
result.

When you measure, write down: the printer model, the driver's paper size and
scaling setting, the nominal and actual length of both rules, the nominal and
actual side of the square, and the date. Measure with a steel rule, not a tape,
and read **centre of stroke to centre of stroke** — the same convention the
lines are drawn under. Reading outer edge to outer edge adds one line width,
about 0.09 mm, to every measurement in the same direction. That is a fifth of
the tolerance spent on nothing, and it is spent silently.

Two printers, not one. A single printer cannot distinguish a driver bug from a
geometry bug, and the whole point of the exercise is that it is able to return
the answer "the software is wrong".
