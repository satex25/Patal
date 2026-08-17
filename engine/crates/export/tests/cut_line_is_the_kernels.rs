//! C11, enforced against the document rather than against the intent.
//!
//! The rule: there is one implementation of where cloth gets cut, it lives in
//! the geometry kernel, and export draws what it is given. Two things enforce
//! it. The [`CutLine`](patal_pattern::CutLine) newtype makes a second
//! implementation *unrepresentable* — this crate cannot construct one, so it
//! cannot substitute its own. These tests cover the other half: that the
//! points which actually reach the page are the ones the kernel produced,
//! moved by nothing but a translation.
//!
//! A scale error is the failure mode most likely to reach cloth and least
//! likely to be caught, because every number involved stays plausible. So it
//! is checked here in absolute terms, against a constant computed from the
//! definition of an inch, and not against anything the crate under test also
//! believes.

mod common;

use common::parse;
use patal_export::{export_tiled_pdf, PageLayout, Pt};
use patal_geometry::{PatternBoundary, Point2};
use patal_pattern::{PatternPiece, Project, DEFAULT_FLATTEN_TOLERANCE_MM};

/// 1mm in PostScript points, from the two integers that define it.
const PT_PER_MM_INDEPENDENT: f64 = 72.0 / 25.4;

/// Wraps loose pieces in a project at the default tolerance.
///
/// Export takes the whole document since D6-A, because the flattening
/// tolerance belongs to the document rather than to whoever is calling
/// export. These tests still build pieces individually — that is what they
/// are about — so this puts them in the container export reads the tolerance
/// from, and nothing here ever sets a non-default one.
fn project_of(pieces: &[&PatternPiece]) -> Project {
    let mut project = Project::new("Export Fixture");
    for piece in pieces {
        project.add_piece((*piece).clone());
    }
    project
}

fn piece(name: &str, points: Vec<(f64, f64)>) -> PatternPiece {
    PatternPiece::from_boundary(
        name,
        PatternBoundary::new(points.into_iter().map(|(x, y)| Point2::new(x, y)).collect())
            .expect("a valid boundary"),
    )
}

fn rect(name: &str, w: f64, h: f64) -> PatternPiece {
    piece(name, vec![(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)])
}

#[test]
fn the_drawn_outline_is_the_kernels_cut_line_under_a_pure_translation() {
    let piece = rect("Front Bodice", 120.0, 160.0);
    let expected = piece
        .cut_boundary(DEFAULT_FLATTEN_TOLERANCE_MM)
        .expect("cuts cleanly");

    let pdf = export_tiled_pdf(&project_of(&[&piece]), &PageLayout::a4()).expect("exports");
    let pages = parse(&pdf);
    assert_eq!(pages.len(), 2, "calibration page plus one sheet");

    let sheet = &pages[1];
    let (origin_x, origin_y) = sheet.tile_origin_mm().expect("the sheet states its origin");
    let (window_x, window_y) = sheet
        .window_origin_mm()
        .expect("the sheet states its window");

    let drawn = sheet.longest_polyline().expect("something was drawn");
    assert_eq!(
        drawn.len(),
        expected.points().len(),
        "every vertex of the cut line must reach the page — none dropped, none invented"
    );

    // Undo the page transform using only what the document declares, then
    // compare against the kernel's own points. Any scale factor, any rounding
    // past four decimal places, any re-flattening shows up here as a
    // millimetre-scale disagreement.
    for (drawn, expected) in drawn.iter().zip(expected.points()) {
        let model_x = drawn.0.get() / PT_PER_MM_INDEPENDENT - window_x.get() + origin_x.get();
        let model_y = drawn.1.get() / PT_PER_MM_INDEPENDENT - window_y.get() + origin_y.get();
        assert!(
            (model_x - expected.x).abs() < 1e-3,
            "x: page says {model_x}, the kernel says {}",
            expected.x
        );
        assert!(
            (model_y - expected.y).abs() < 1e-3,
            "y: page says {model_y}, the kernel says {}",
            expected.y
        );
    }
}

#[test]
fn a_two_hundred_millimetre_edge_is_two_hundred_millimetres_on_the_page() {
    // The scale test, stated the way a steel rule states it. A piece with a
    // known 200mm edge must produce a 200mm edge in page space. This is what
    // catches a factor-of-25.4 error, a doubled conversion, or a stray `cm`
    // matrix — none of which the round-trip test above can see, because that
    // one undoes the transform using the same constant it was applied with.
    let piece = rect("Ruler", 200.0, 100.0);
    let cut = piece
        .cut_boundary(DEFAULT_FLATTEN_TOLERANCE_MM)
        .expect("cuts cleanly");
    // The cut line of a rectangle with a 10mm allowance is 220 x 120.
    let width_mm = 220.0;

    let pdf = export_tiled_pdf(&project_of(&[&piece]), &PageLayout::a4()).expect("exports");
    let pages = parse(&pdf);
    let drawn = pages[1].longest_polyline().expect("something was drawn");

    let min_x = drawn.iter().map(|p| p.0.get()).fold(f64::MAX, f64::min);
    let max_x = drawn.iter().map(|p| p.0.get()).fold(f64::MIN, f64::max);
    let span_pt = max_x - min_x;
    let expected_pt = width_mm * PT_PER_MM_INDEPENDENT; // 623.62 pt

    assert!(
        (span_pt - expected_pt).abs() < 1e-2,
        "a {width_mm}mm edge came out {span_pt}pt; at true scale it must be {expected_pt}pt"
    );
    // And the kernel agrees the edge is that wide in the first place.
    let model_span = cut.points().iter().map(|p| p.x).fold(f64::MIN, f64::max)
        - cut.points().iter().map(|p| p.x).fold(f64::MAX, f64::min);
    assert!((model_span - width_mm).abs() < 1e-9);
}

#[test]
fn the_calibration_square_is_exactly_fifty_millimetres() {
    // C12. The claim printed on the page has to be true of the page.
    let piece = rect("Front", 100.0, 100.0);
    let pdf = export_tiled_pdf(&project_of(&[&piece]), &PageLayout::a4()).expect("exports");
    let pages = parse(&pdf);

    let expected = 50.0 * PT_PER_MM_INDEPENDENT; // 141.7322835 pt
    for (index, page) in pages.iter().enumerate().skip(1) {
        let square = page
            .rects
            .iter()
            .find(|(_, _, w, h)| (w - expected).abs() < 1e-2 && (h - expected).abs() < 1e-2);
        assert!(
            square.is_some(),
            "sheet {index} has no 50mm ({expected}pt) square; it has {:?}",
            page.rects
        );
    }
}

#[test]
fn the_calibration_square_never_lands_on_a_cut_line() {
    // F7: the square is a reserved strip, not an overlay. A 50mm box drawn on
    // top of a piece would, on some sheet, sit across the line someone is
    // about to cut — and it is labelled "measure me", so they would.
    let piece = rect("Wide", 600.0, 700.0);
    let layout = PageLayout::a4();
    let pdf = export_tiled_pdf(&project_of(&[&piece]), &layout).expect("exports");
    let pages = parse(&pdf);

    let strip_top = (layout.margin() + patal_export::Mm(patal_export::CALIBRATION_STRIP_MM))
        .to_pt()
        .get();

    for (index, page) in pages.iter().enumerate().skip(1) {
        // The guarantee is structural, so this is what to assert: pattern
        // geometry is drawn inside a clip whose floor is the top of the
        // strip. Checking the drawn points instead would prove nothing —
        // a piece taller than one sheet legitimately has vertices hundreds of
        // points below the page, and the clip is exactly what stops them
        // reaching the paper.
        assert_eq!(
            page.clips.len(),
            1,
            "sheet {index} should clip pattern geometry exactly once"
        );
        let clip = page.clips[0];
        assert!(
            clip.1 >= strip_top - 1e-3,
            "sheet {index} clips from {}pt, below the strip top at {strip_top}pt — \
             a cut line could be drawn across the square someone is told to measure",
            clip.1
        );

        // And the square itself is inside the strip, below that floor.
        let expected = 50.0 * PT_PER_MM_INDEPENDENT;
        let square = page
            .rects
            .iter()
            .find(|(_, _, w, h)| (w - expected).abs() < 1e-2 && (h - expected).abs() < 1e-2)
            .unwrap_or_else(|| panic!("sheet {index} has no calibration square"));
        assert!(
            square.1 + square.3 <= strip_top + 1e-3,
            "sheet {index} puts the calibration square outside the reserved strip"
        );
    }
}

#[test]
fn a_piece_drafted_downward_from_the_origin_still_tiles_correctly() {
    // The harness's demo bodice runs from y = 0 down to y = -420, which is
    // the natural way to draft downward from a shoulder point. Model
    // coordinates are not page coordinates, and nothing may assume they are
    // positive — a `min` mistaken for a zero puts the whole piece off the
    // paper, and every individual page still looks like a valid PDF.
    let piece = piece(
        "Bodice Front",
        vec![(0.0, 0.0), (150.0, 0.0), (150.0, -420.0), (0.0, -420.0)],
    );
    let cut = piece
        .cut_boundary(DEFAULT_FLATTEN_TOLERANCE_MM)
        .expect("cuts cleanly");
    let min_x = cut.points().iter().map(|p| p.x).fold(f64::MAX, f64::min);
    let min_y = cut.points().iter().map(|p| p.y).fold(f64::MAX, f64::min);

    let pdf = export_tiled_pdf(&project_of(&[&piece]), &PageLayout::a4()).expect("exports");
    let pages = parse(&pdf);

    // Some sheet must start exactly at the piece's bottom-left corner,
    // negative y and all, or the bottom of the pattern is never printed.
    let origins: Vec<(f64, f64)> = pages[1..]
        .iter()
        .map(|p| {
            let (x, y) = p.tile_origin_mm().expect("every sheet states its origin");
            (x.get(), y.get())
        })
        .collect();
    assert!(
        origins
            .iter()
            .any(|(x, y)| (x - min_x).abs() < 1e-6 && (y - min_y).abs() < 1e-6),
        "no sheet starts at the piece's corner ({min_x}, {min_y}); origins are {origins:?}"
    );

    // And every sheet still reconstructs the same outline.
    for sheet in &pages[1..] {
        let (ox, oy) = sheet.tile_origin_mm().unwrap();
        let (wx, wy) = sheet.window_origin_mm().unwrap();
        let drawn = sheet.longest_polyline().expect("something was drawn");
        for (drawn, expected) in drawn.iter().zip(cut.points()) {
            let model_y = drawn.1.get() / PT_PER_MM_INDEPENDENT - wy.get() + oy.get();
            let model_x = drawn.0.get() / PT_PER_MM_INDEPENDENT - wx.get() + ox.get();
            assert!((model_x - expected.x).abs() < 1e-3);
            assert!((model_y - expected.y).abs() < 1e-3);
        }
    }
}

#[test]
fn the_sewing_line_and_the_cut_line_are_both_drawn_and_are_different() {
    // A sewer needs both, and needs to tell them apart. The cut line is the
    // outer one for a positive allowance.
    let piece = rect("Front", 100.0, 100.0);
    let pdf = export_tiled_pdf(&project_of(&[&piece]), &PageLayout::a4()).expect("exports");
    let pages = parse(&pdf);

    let closed: Vec<&Vec<(Pt, Pt)>> = pages[1].polylines.iter().filter(|p| p.len() == 4).collect();
    assert_eq!(closed.len(), 2, "sewing line and cut line");

    let extent = |p: &Vec<(Pt, Pt)>| {
        let max = p.iter().map(|q| q.0.get()).fold(f64::MIN, f64::max);
        let min = p.iter().map(|q| q.0.get()).fold(f64::MAX, f64::min);
        max - min
    };
    let allowance_pt = 2.0 * 10.0 * PT_PER_MM_INDEPENDENT;
    assert!(
        (extent(closed[1]) - extent(closed[0]) - allowance_pt).abs() < 1e-2,
        "the cut line must be a 10mm seam allowance outside the sewing line"
    );
}
