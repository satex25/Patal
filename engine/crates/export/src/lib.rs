//! Tiled, true-scale PDF export for Pātāl patterns.
//!
//! This crate turns pattern pieces into sheets of A4 or Letter that, printed
//! at 100% and taped together, are the pattern. It is the first code in the
//! project whose output is checked by a ruler rather than an assertion.
//!
//! # The three rules this crate exists to keep
//!
//! **One implementation of the cut line.** Export never computes where cloth
//! gets cut. It asks [`PatternPiece::cut_boundary`] and draws what comes
//! back. That is not a convention here, it is a type: a [`CutLine`] has a
//! private field and no public
//! constructor, so this crate *cannot* fabricate one however convenient it
//! would be. A second piece of code deciding where the scissors go is the
//! defect that got the Swift offset kernel deleted, and it is worth spending
//! a newtype to make it unrepresentable.
//!
//! **True scale, or nothing.** A millimetre in the model is a millimetre on
//! the paper. There is no scale parameter and no fit-to-page. Every page
//! carries a 50mm calibration square in a reserved strip so the claim is
//! checkable on the artifact itself, with a rule, by someone who does not
//! trust it.
//!
//! **No partial documents.** Every cut line is derived before a single byte
//! is written. A six-piece PDF produced because the seventh piece failed
//! looks exactly like a correct one, and the person who discovers otherwise
//! is holding scissors.
//!
//! # Not in this crate
//!
//! No file I/O. [`export_tiled_pdf`] returns bytes and the caller writes
//! them — via a temporary file and a rename, so that an I/O failure halfway
//! through cannot leave a truncated PDF that still opens and still prints.
//! Keeping `std::fs` out also keeps the core free of the platform, per
//! ADR-001.
//!
//! No nesting or lay planning. Each piece gets its own tile grid at its own
//! origin. Packing several pieces onto shared sheets is two-dimensional bin
//! packing with grain constraints — a different problem from a page
//! transform, deferred deliberately.
//!
//! # Example
//!
//! ```
//! use patal_export::{export_tiled_pdf, PageLayout};
//! use patal_geometry::{PatternBoundary, Point2};
//! use patal_pattern::PatternPiece;
//!
//! let boundary = PatternBoundary::new(vec![
//!     Point2::new(0.0, 0.0),
//!     Point2::new(200.0, 0.0),
//!     Point2::new(200.0, 300.0),
//!     Point2::new(0.0, 300.0),
//! ])?;
//! let piece = PatternPiece::new("Front Bodice", boundary);
//!
//! let pdf = export_tiled_pdf(&[&piece], &PageLayout::a4())?;
//! assert!(pdf.starts_with(b"%PDF-1.7"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![forbid(unsafe_code)]

mod error;
mod layout;
mod pdf;
mod units;

pub use error::ExportError;
pub use layout::{
    BoundsMm, PageLayout, TileGrid, CALIBRATION_SQUARE_MM, CALIBRATION_STRIP_MM, DEFAULT_MARGIN_MM,
    DEFAULT_OVERLAP_MM, MAX_TILES_PER_PIECE, MIN_MARGIN_MM,
};
pub use pdf::HAIRLINE_PT;
pub use units::{Mm, Pt, MM_PER_PT, PT_PER_MM};

use patal_pattern::{CutLine, PatternPiece};

use pdf::{Canvas, Page};

/// Everything derived from one piece before any drawing happens.
struct PiecePlan {
    name: String,
    seam_allowance_mm: f64,
    /// The line the scissors follow. Minted by `patal-pattern`; this crate
    /// can read it and cannot make one.
    cut: CutLine,
    /// The authored outline — the *sewing* line. Drawn dashed, because a
    /// sewer needs both and being unable to tell them apart is how a garment
    /// comes out a seam allowance too small all round.
    ///
    /// Not a second cut line: this is the piece's own stored boundary, not a
    /// re-derivation of the offset.
    sewing: patal_geometry::PatternBoundary,
    grid: TileGrid,
}

/// Lays pieces out across tiled pages and returns a complete PDF.
///
/// Each piece gets its own grid of sheets; sheets within a piece overlap by
/// `layout.overlap()` and carry shared registration crosses on the model grid
/// lines, so aligning two sheets by their crosses is the same operation as
/// assembling the piece correctly.
///
/// # Errors
///
/// Returns [`ExportError`] and no bytes at all if the pieces are empty, if a
/// piece has no valid cut line, or if a piece needs more sheets than
/// [`MAX_TILES_PER_PIECE`]. Page geometry is validated earlier, when the
/// [`PageLayout`] is constructed.
pub fn export_tiled_pdf(
    pieces: &[&PatternPiece],
    layout: &PageLayout,
) -> Result<Vec<u8>, ExportError> {
    if pieces.is_empty() {
        return Err(ExportError::NothingToExport);
    }

    // Phase one: derive everything, emit nothing. If any piece fails, the
    // caller gets an error and no document — not a document missing a piece.
    let mut plans: Vec<PiecePlan> = Vec::with_capacity(pieces.len());
    for piece in pieces {
        let cut = piece
            .cut_boundary()
            .map_err(|source| ExportError::CutLineFailed {
                piece: piece.name.clone(),
                source,
            })?;

        // The drawn extent is the union of both lines. Bounding only the cut
        // line would be wrong for a negative allowance, where the sewing line
        // is the outer of the two.
        let bounds =
            BoundsMm::of_boundary(cut.boundary()).union(BoundsMm::of_boundary(&piece.boundary));
        let grid = TileGrid::cover(bounds, *layout);

        if grid.tiles() > MAX_TILES_PER_PIECE {
            return Err(ExportError::TooManyTiles {
                piece: piece.name.clone(),
                tiles: grid.tiles(),
                maximum: MAX_TILES_PER_PIECE,
            });
        }

        plans.push(PiecePlan {
            name: piece.name.clone(),
            seam_allowance_mm: piece.seam_allowance_mm(),
            cut,
            sewing: piece.boundary.clone(),
            grid,
        });
    }

    // Phase two: draw.
    let mut pages = vec![calibration_page(layout, &plans)];
    for plan in &plans {
        // Rows top-down, so sheet 1 of a piece is its top-left corner and the
        // stack comes out of the printer in the order it gets taped.
        for row in (0..plan.grid.rows).rev() {
            for column in 0..plan.grid.columns {
                pages.push(tile_page(layout, plan, column, row));
            }
        }
    }

    Ok(pdf::render(&pages))
}

/// Converts a page-space millimetre coordinate into the points the content
/// stream speaks. The only place in this module where units change.
fn at(x: Mm, y: Mm) -> (Pt, Pt) {
    (x.to_pt(), y.to_pt())
}

/// Draws the calibration square and the sheet's labels in the reserved strip.
///
/// The strip is reserved, not overlaid, so this cannot collide with an
/// outline: the drawable window starts above it, and every model point is
/// clipped to that window.
fn calibration_strip(canvas: &mut Canvas, layout: &PageLayout, labels: &[String]) {
    let margin = layout.margin();
    let square = Mm(CALIBRATION_SQUARE_MM);

    canvas.save();
    canvas.grey(0.0).line_width_pt(HAIRLINE_PT);
    canvas.dash(&[3.0, 2.0]);
    let (x, y) = at(margin, margin);
    canvas.rect(x, y, square.to_pt(), square.to_pt());
    canvas.dash(&[]);

    // Ticks on two sides so the measurement has unambiguous endpoints — the
    // outside of the dashed line is not the same place as the inside, and at
    // 0.25pt that is a tenth of a millimetre of argument.
    let text_x = margin + square + Mm(6.0);
    let mut baseline = margin + square - Mm(4.0);
    canvas.text(
        at(text_x, baseline).0,
        at(text_x, baseline).1,
        9.0,
        "50mm calibration square - DO NOT CUT",
    );
    baseline = baseline - Mm(6.0);
    canvas.text(
        at(text_x, baseline).0,
        at(text_x, baseline).1,
        7.0,
        "Measure it. If it is not 50mm, the print is scaled and the pattern is wrong.",
    );
    for label in labels {
        baseline = baseline - Mm(6.0);
        canvas.text(at(text_x, baseline).0, at(text_x, baseline).1, 7.0, label);
    }
    canvas.restore();
}

/// The first page: the long baselines that a 50mm square is too short to
/// catch.
///
/// A 1% scale error moves a 50mm square by half a millimetre, which is inside
/// the noise of reading a steel rule. Over 200mm the same error moves the
/// mark by two millimetres, which is not. Both axes are drawn because printer
/// scaling differs between the feed direction and the carriage direction, and
/// a single rule cannot tell those apart.
fn calibration_page(layout: &PageLayout, plans: &[PiecePlan]) -> Page {
    let margin = layout.margin();
    let mut canvas = Canvas::new();
    canvas.comment("patal-calibration-page");
    canvas.grey(0.0).line_width_pt(HAIRLINE_PT);

    let top = layout.page_height() - margin;
    let mut baseline = top - Mm(6.0);
    canvas.text(
        at(margin, baseline).0,
        at(margin, baseline).1,
        14.0,
        "Patal - true-scale check",
    );

    for line in [
        "Print at 100%. Turn off 'fit to page', 'shrink oversized pages' and any scaling.",
        "Set the printer driver's paper size to the same size as this PDF - a Letter tray",
        "printing an A4 document silently shrinks it by about 6%, and no scaling setting",
        "in the print dialog will say so.",
        "Then measure the two rules below from the corner, and the square on every sheet.",
    ] {
        baseline = baseline - Mm(5.5);
        canvas.text(at(margin, baseline).0, at(margin, baseline).1, 9.0, line);
    }

    // The L. One origin, two axes, so both are measured from the same corner.
    let origin_x = margin + Mm(4.0);
    let origin_y = margin + Mm(4.0);
    let span_x = Mm(200.0).min(layout.page_width() - margin - origin_x);
    let span_y = Mm(200.0).min(baseline - Mm(14.0) - origin_y);

    canvas.line(at(origin_x, origin_y), at(origin_x + span_x, origin_y));
    canvas.line(at(origin_x, origin_y), at(origin_x, origin_y + span_y));
    // End ticks, so "the end of the line" is a defined place.
    canvas.line(
        at(origin_x + span_x, origin_y - Mm(3.0)),
        at(origin_x + span_x, origin_y + Mm(3.0)),
    );
    canvas.line(
        at(origin_x - Mm(3.0), origin_y + span_y),
        at(origin_x + Mm(3.0), origin_y + span_y),
    );
    canvas.line(
        at(origin_x, origin_y - Mm(3.0)),
        at(origin_x, origin_y + Mm(3.0)),
    );
    canvas.line(
        at(origin_x - Mm(3.0), origin_y),
        at(origin_x + Mm(3.0), origin_y),
    );

    canvas.text(
        at(origin_x + Mm(3.0), origin_y + Mm(3.0)).0,
        at(origin_x + Mm(3.0), origin_y + Mm(3.0)).1,
        8.0,
        &format!(
            "across the page: {} mm from this corner to the tick",
            trim(span_x.get())
        ),
    );
    canvas.text(
        at(origin_x + Mm(5.0), origin_y + span_y - Mm(6.0)).0,
        at(origin_x + Mm(5.0), origin_y + span_y - Mm(6.0)).1,
        8.0,
        &format!(
            "down the page: {} mm from the corner to this tick",
            trim(span_y.get())
        ),
    );

    // Provenance, printed rather than filed: the artifact is the log for a
    // tool whose output someone cuts cloth with.
    let mut info = vec![
        format!(
            "page {} x {} mm, margin {} mm, overlap {} mm",
            trim(layout.page_width().get()),
            trim(layout.page_height().get()),
            trim(margin.get()),
            trim(layout.overlap().get())
        ),
        format!("{} piece(s), no nesting - one grid per piece", plans.len()),
    ];
    for plan in plans {
        info.push(format!(
            "  {}: {} x {} sheets, seam allowance {} mm",
            plan.name,
            plan.grid.columns,
            plan.grid.rows,
            trim(plan.seam_allowance_mm)
        ));
    }

    let mut y = origin_y + span_y - Mm(20.0);
    let x = origin_x + Mm(70.0);
    for line in &info {
        canvas.text(at(x, y).0, at(x, y).1, 8.0, line);
        y = y - Mm(5.0);
    }

    Page {
        width: layout.page_width().to_pt(),
        height: layout.page_height().to_pt(),
        canvas,
    }
}

/// One sheet of one piece.
fn tile_page(layout: &PageLayout, plan: &PiecePlan, column: usize, row: usize) -> Page {
    let (origin_x, origin_y) = plan.grid.tile_origin(column, row);
    let (window_x, window_y) = layout.window_origin();
    let window_w = layout.window_width();
    let window_h = layout.window_height();

    // The page transform, in full. A model point moves by the difference
    // between where this sheet's window starts on the page and where it
    // starts in the model — a translation and nothing else. No rotation, no
    // scale, nothing that could quietly resize a pattern.
    let place = |x: f64, y: f64| -> (Pt, Pt) {
        at(Mm(x) - origin_x + window_x, Mm(y) - origin_y + window_y)
    };

    let mut canvas = Canvas::new();
    canvas.comment(&format!(
        "patal-tile piece={:?} col={} row={} columns={} rows={} \
         origin_mm={},{} window_mm={},{} window_origin_mm={},{}",
        plan.name,
        column,
        row,
        plan.grid.columns,
        plan.grid.rows,
        trim(origin_x.get()),
        trim(origin_y.get()),
        trim(window_w.get()),
        trim(window_h.get()),
        trim(window_x.get()),
        trim(window_y.get()),
    ));

    canvas.save();
    canvas.clip_rect(
        window_x.to_pt(),
        window_y.to_pt(),
        window_w.to_pt(),
        window_h.to_pt(),
    );
    canvas.line_width_pt(HAIRLINE_PT);

    // Sewing line first, dashed and grey, so the solid black line on top is
    // unambiguously the one to cut.
    canvas.grey(0.45).dash(&[4.0, 3.0]);
    let sewing: Vec<(Pt, Pt)> = plan
        .sewing
        .points()
        .iter()
        .map(|p| place(p.x, p.y))
        .collect();
    canvas.polyline(&sewing, true);

    // The cut line. These points come from `CutLine` and are not recomputed
    // here — see the crate docs.
    canvas.grey(0.0).dash(&[]);
    let cut: Vec<(Pt, Pt)> = plan.cut.points().iter().map(|p| place(p.x, p.y)).collect();
    canvas.polyline(&cut, true);

    // Registration crosses on the shared model grid lines. Two sheets that
    // overlap carry the same cross at the same model coordinate.
    canvas.grey(0.0);
    let arm = 4.0;
    for (i, gx) in plan.grid.grid_x().iter().enumerate() {
        for (j, gy) in plan.grid.grid_y().iter().enumerate() {
            let inside_x =
                gx.get() >= origin_x.get() - 1e-9 && gx.get() <= (origin_x + window_w).get() + 1e-9;
            let inside_y =
                gy.get() >= origin_y.get() - 1e-9 && gy.get() <= (origin_y + window_h).get() + 1e-9;
            if !(inside_x && inside_y) {
                continue;
            }
            canvas.line(
                place(gx.get() - arm, gy.get()),
                place(gx.get() + arm, gy.get()),
            );
            canvas.line(
                place(gx.get(), gy.get() - arm),
                place(gx.get(), gy.get() + arm),
            );
            canvas.text(
                place(gx.get() + 1.0, gy.get() + 1.0).0,
                place(gx.get() + 1.0, gy.get() + 1.0).1,
                6.0,
                &format!("x{i}y{j}"),
            );
        }
    }
    canvas.restore();

    // The window edge, drawn outside the clip so it is not clipped by itself.
    canvas.save();
    canvas
        .grey(0.7)
        .line_width_pt(HAIRLINE_PT)
        .dash(&[2.0, 2.0]);
    canvas.rect(
        window_x.to_pt(),
        window_y.to_pt(),
        window_w.to_pt(),
        window_h.to_pt(),
    );
    canvas.restore();

    let sheet = row_major_sheet_number(plan, column, row);
    calibration_strip(
        &mut canvas,
        layout,
        &[
            format!(
                "{}  -  sheet {} of {}  (column {} of {}, row {} of {})",
                plan.name,
                sheet,
                plan.grid.tiles(),
                column + 1,
                plan.grid.columns,
                plan.grid.rows - row,
                plan.grid.rows
            ),
            format!(
                "solid = cut line (seam allowance {} mm).  dashed grey = sewing line.",
                trim(plan.seam_allowance_mm)
            ),
            "Assembly: overlap the crosses, do not trim. Matching labels (x1y0) must coincide."
                .to_string(),
        ],
    );

    Page {
        width: layout.page_width().to_pt(),
        height: layout.page_height().to_pt(),
        canvas,
    }
}

/// Sheet numbering in the order the pages come out: top row first.
fn row_major_sheet_number(plan: &PiecePlan, column: usize, row: usize) -> usize {
    let from_top = plan.grid.rows - 1 - row;
    from_top * plan.grid.columns + column + 1
}

/// Renders a millimetre value for a human-readable label.
fn trim(value: f64) -> String {
    let mut s = format!("{value:.2}");
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use patal_geometry::{PatternBoundary, Point2};

    fn rect_piece(name: &str, w: f64, h: f64) -> PatternPiece {
        PatternPiece::new(
            name,
            PatternBoundary::new(vec![
                Point2::new(0.0, 0.0),
                Point2::new(w, 0.0),
                Point2::new(w, h),
                Point2::new(0.0, h),
            ])
            .expect("a rectangle is a valid boundary"),
        )
    }

    #[test]
    fn exporting_nothing_is_an_error_not_an_empty_document() {
        assert_eq!(
            export_tiled_pdf(&[], &PageLayout::a4()),
            Err(ExportError::NothingToExport)
        );
    }

    #[test]
    fn a_small_piece_produces_a_calibration_page_and_one_sheet() {
        let piece = rect_piece("Front Bodice", 100.0, 150.0);
        let pdf = export_tiled_pdf(&[&piece], &PageLayout::a4()).expect("exports");
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("/Count 2"), "calibration page plus one sheet");
        assert!(text.contains("patal-calibration-page"));
        assert!(text.contains("col=0 row=0 columns=1 rows=1"));
    }

    /// A square with a deep, narrow V notch cut into its top edge — the
    /// classic dart geometry, and the shape that actually fails.
    ///
    /// A convex outline under a positive offset simply gets bigger, however
    /// absurd the allowance; a 20mm square with a 500mm seam allowance is a
    /// perfectly valid 1020mm square. The failure this crate has to survive
    /// needs a concave feature narrower than twice the offset, where the two
    /// walls cross rather than the notch vanishing.
    ///
    /// 5mm is the allowance that breaks this particular notch. Larger ones
    /// pass, because past about 10mm the notch is consumed entirely and the
    /// mitre limit bevels what is left into a clean outline — which is worth
    /// knowing when picking a value here, and is why this is a named fixture
    /// rather than an inline literal.
    fn notched_piece(name: &str) -> PatternPiece {
        PatternPiece::new(
            name,
            PatternBoundary::new(vec![
                Point2::new(0.0, 0.0),
                Point2::new(100.0, 0.0),
                Point2::new(100.0, 100.0),
                Point2::new(55.0, 100.0),
                Point2::new(50.0, 20.0),
                Point2::new(45.0, 100.0),
                Point2::new(0.0, 100.0),
            ])
            .expect("a notched square is a valid boundary"),
        )
    }

    #[test]
    fn a_failing_piece_yields_no_bytes_at_all() {
        // The seventh-piece problem: a document that comes back
        // six-sevenths correct looks exactly like one that is right.
        let good = rect_piece("Front", 100.0, 100.0);
        let mut bad = notched_piece("Back");
        bad.set_seam_allowance_mm(5.0).expect("finite and positive");

        let err = export_tiled_pdf(&[&good, &bad], &PageLayout::a4()).unwrap_err();
        match err {
            ExportError::CutLineFailed { piece, .. } => assert_eq!(piece, "Back"),
            other => panic!("expected the failure to name the piece, got {other:?}"),
        }
    }

    #[test]
    fn a_piece_in_the_wrong_units_is_refused_rather_than_printed() {
        // 2000 x 3000 "mm" is what a piece authored in the wrong units looks
        // like. On A4 that is well past four hundred sheets.
        let piece = rect_piece("Oops", 20_000.0, 30_000.0);
        let err = export_tiled_pdf(&[&piece], &PageLayout::a4()).unwrap_err();
        assert!(matches!(err, ExportError::TooManyTiles { .. }), "{err:?}");
        assert!(err.to_string().contains("check the piece's units"), "{err}");
    }

    #[test]
    fn a_coordinate_past_any_paper_is_refused_rather_than_panicking() {
        // The sibling test above uses 20m, which a person could plausibly
        // type. This one uses a billion kilometres, which they could not —
        // it is what a corrupt file or a bad import looks like, and
        // `PatternBoundary` accepts it because it is finite.
        //
        // The piece still has a valid cut line, so the refusal has to come
        // from the tile count, which is where the arithmetic used to give
        // out before the check that reads it ever ran.
        let piece = rect_piece("Corrupt", 1.0e12, 1.0e12);
        let err = export_tiled_pdf(&[&piece], &PageLayout::a4()).unwrap_err();
        assert!(matches!(err, ExportError::TooManyTiles { .. }), "{err:?}");
        assert!(err.to_string().contains("check the piece's units"), "{err}");
    }

    #[test]
    fn every_sheet_carries_a_calibration_square() {
        let piece = rect_piece("Wide", 600.0, 500.0);
        let pdf = export_tiled_pdf(&[&piece], &PageLayout::a4()).expect("exports");
        let text = String::from_utf8_lossy(&pdf);
        let pages = text.matches("/Type /Page ").count();
        let squares = text.matches("50mm calibration square").count();
        assert!(pages >= 5, "expected a tiled piece, got {pages} pages");
        assert_eq!(
            squares,
            pages - 1,
            "every sheet but the calibration page itself carries the square"
        );
    }

    #[test]
    fn several_pieces_each_get_their_own_grid() {
        let a = rect_piece("Front", 100.0, 100.0);
        let b = rect_piece("Back", 100.0, 100.0);
        let pdf = export_tiled_pdf(&[&a, &b], &PageLayout::a4()).expect("exports");
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("piece=\"Front\""));
        assert!(text.contains("piece=\"Back\""));
        assert!(text.contains("/Count 3"));
    }

    #[test]
    fn the_same_pieces_always_export_to_the_same_bytes() {
        let piece = rect_piece("Front Bodice", 250.0, 400.0);
        let once = export_tiled_pdf(&[&piece], &PageLayout::a4()).unwrap();
        let twice = export_tiled_pdf(&[&piece], &PageLayout::a4()).unwrap();
        assert_eq!(once, twice, "export must be a pure function of its input");
    }

    #[test]
    fn letter_and_a4_differ_in_the_media_box() {
        let piece = rect_piece("Front", 100.0, 100.0);
        let a4 = String::from_utf8_lossy(&export_tiled_pdf(&[&piece], &PageLayout::a4()).unwrap())
            .to_string();
        let letter =
            String::from_utf8_lossy(&export_tiled_pdf(&[&piece], &PageLayout::letter()).unwrap())
                .to_string();
        assert!(
            a4.contains("/MediaBox [0 0 595.2756 841.8898]"),
            "A4 in points"
        );
        assert!(
            letter.contains("/MediaBox [0 0 612 792]"),
            "Letter is exactly 612 x 792 pt"
        );
    }
}
