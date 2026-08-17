//! Assembly, tested the way the operator does it.
//!
//! Per-page tests pass happily while the assembled pattern is wrong. An
//! off-by-one in the tile count drops a strip of the piece; an overlap
//! applied twice grows it by `overlap x (rows - 1)`; a translation that uses
//! the wrong row index puts a correct drawing on the wrong sheet. Every one
//! of those produces pages that individually look right.
//!
//! So these tests reassemble. Each sheet declares, in its own content stream,
//! the model coordinate its window starts at. Undo that translation on the
//! points the sheet actually draws and every sheet must reconstruct the same
//! global outline — which is exactly what taping the crosses together does,
//! done in arithmetic.

mod common;

use common::{parse, ParsedPage};
use patal_export::{export_tiled_pdf, PageLayout};
use patal_geometry::{PatternBoundary, Point2};
use patal_pattern::{PatternPiece, Project, DEFAULT_FLATTEN_TOLERANCE_MM};

const PT_PER_MM: f64 = 72.0 / 25.4;

/// Wraps loose pieces in a project at the default tolerance. See the same
/// helper in `cut_line_is_the_kernels` for why export takes a document.
fn project_of(pieces: &[&PatternPiece]) -> Project {
    let mut project = Project::new("Export Fixture");
    for piece in pieces {
        project.add_piece((*piece).clone());
    }
    project
}

fn rect(name: &str, w: f64, h: f64) -> PatternPiece {
    PatternPiece::from_boundary(
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

/// The sheet's longest polyline, put back into model millimetres using only
/// what the sheet itself declares.
fn reassemble(page: &ParsedPage) -> Vec<(f64, f64)> {
    let (origin_x, origin_y) = page.tile_origin_mm().expect("the sheet states its origin");
    let (window_x, window_y) = page
        .window_origin_mm()
        .expect("the sheet states its window");
    page.longest_polyline()
        .expect("the sheet drew something")
        .iter()
        .map(|(x, y)| {
            (
                x.get() / PT_PER_MM - window_x.get() + origin_x.get(),
                y.get() / PT_PER_MM - window_y.get() + origin_y.get(),
            )
        })
        .collect()
}

#[test]
fn every_sheet_reconstructs_the_same_outline() {
    // A piece needing a 4 x 4 grid on A4: 600 x 700mm of cut line against a
    // 190 x 219mm window advancing 180 x 209mm.
    let piece = rect("Big Panel", 600.0, 700.0);
    let expected = piece
        .cut_boundary(DEFAULT_FLATTEN_TOLERANCE_MM)
        .expect("cuts cleanly");

    let pdf = export_tiled_pdf(&project_of(&[&piece]), &PageLayout::a4()).expect("exports");
    let pages = parse(&pdf);
    let sheets = &pages[1..];
    assert!(
        sheets.len() >= 9,
        "expected a real grid, got {}",
        sheets.len()
    );

    for (index, sheet) in sheets.iter().enumerate() {
        let model = reassemble(sheet);
        assert_eq!(
            model.len(),
            expected.points().len(),
            "sheet {index} draws a different number of vertices"
        );
        for (drawn, expected) in model.iter().zip(expected.points()) {
            assert!(
                (drawn.0 - expected.x).abs() < 1e-3 && (drawn.1 - expected.y).abs() < 1e-3,
                "sheet {index} places a vertex at {drawn:?}, but it belongs at \
                 ({}, {}) — the sheets do not agree on where the piece is",
                expected.x,
                expected.y
            );
        }
    }
}

#[test]
fn the_windows_cover_the_piece_with_no_gap_and_the_stated_overlap() {
    // The off-by-one that silently drops a strip of the pattern. Checked
    // against the origins the document declares, not against the grid code.
    let layout = PageLayout::a4();
    let piece = rect("Big Panel", 600.0, 700.0);
    let cut = piece
        .cut_boundary(DEFAULT_FLATTEN_TOLERANCE_MM)
        .expect("cuts cleanly");
    let pdf = export_tiled_pdf(&project_of(&[&piece]), &layout).expect("exports");
    let pages = parse(&pdf);

    let min_x = cut.points().iter().map(|p| p.x).fold(f64::MAX, f64::min);
    let max_x = cut.points().iter().map(|p| p.x).fold(f64::MIN, f64::max);
    let min_y = cut.points().iter().map(|p| p.y).fold(f64::MAX, f64::min);
    let max_y = cut.points().iter().map(|p| p.y).fold(f64::MIN, f64::max);

    let mut xs: Vec<f64> = Vec::new();
    let mut ys: Vec<f64> = Vec::new();
    for sheet in &pages[1..] {
        let (ox, oy) = sheet.tile_origin_mm().unwrap();
        if !xs.iter().any(|v| (v - ox.get()).abs() < 1e-9) {
            xs.push(ox.get());
        }
        if !ys.iter().any(|v| (v - oy.get()).abs() < 1e-9) {
            ys.push(oy.get());
        }
    }
    xs.sort_by(f64::total_cmp);
    ys.sort_by(f64::total_cmp);

    let check = |starts: &[f64], window: f64, low: f64, high: f64, axis: &str| {
        assert!(
            (starts[0] - low).abs() < 1e-6,
            "{axis}: the first window starts at {}, the piece at {low}",
            starts[0]
        );
        let last_reach = starts[starts.len() - 1] + window;
        assert!(
            last_reach >= high - 1e-6,
            "{axis}: the last window reaches {last_reach}, the piece ends at {high} — \
             a strip of the pattern is never printed"
        );
        for pair in starts.windows(2) {
            let shared = pair[0] + window - pair[1];
            assert!(
                (shared - 10.0).abs() < 1e-6,
                "{axis}: consecutive sheets share {shared}mm, not the 10mm overlap"
            );
        }
    };

    check(&xs, layout.window_width().get(), min_x, max_x, "across");
    check(&ys, layout.window_height().get(), min_y, max_y, "down");
}

/// One sheet's claim about where a labelled cross sits, in model millimetres.
#[derive(Debug, Clone, Copy)]
struct Mark {
    sheet: usize,
    x: f64,
    y: f64,
}

/// Where a sheet says the cross labelled `xNyM` sits, in model millimetres.
///
/// The label is drawn 1mm up and right of the cross it names (see
/// `tile_page`), so undoing that offset recovers the marked point from the
/// document alone.
fn labelled_marks(sheet: &ParsedPage) -> Vec<(String, f64, f64)> {
    sheet
        .placed_texts
        .iter()
        .filter(|(_, _, label)| {
            label.starts_with('x') && label.contains('y') && !label.contains(' ')
        })
        .filter_map(|(x, y, label)| {
            let (mx, my) = sheet.to_model_mm(*x, *y)?;
            Some((label.clone(), mx.get() - 1.0, my.get() - 1.0))
        })
        .collect()
}

#[test]
fn overlapping_sheets_carry_the_same_registration_marks() {
    // The assembly contract, and it is printed on every sheet: "Matching
    // labels (x1y0) must coincide." So the test has to be about labels. Two
    // sheets that share a mark at the same model point but disagree about
    // what to call it are two sheets an operator cannot align, and a test
    // that only counts shared positions would pass on exactly that document.
    let piece = rect("Big Panel", 600.0, 300.0);
    let pdf = export_tiled_pdf(&project_of(&[&piece]), &PageLayout::a4()).expect("exports");
    let pages = parse(&pdf);
    let sheets = &pages[1..];
    assert!(sheets.len() >= 4, "expected a grid, got {}", sheets.len());

    // label -> every sheet that claims it, and where.
    let mut claims: Vec<(String, Vec<Mark>)> = Vec::new();
    for (sheet_index, sheet) in sheets.iter().enumerate() {
        for (label, x, y) in labelled_marks(sheet) {
            let mark = Mark {
                sheet: sheet_index,
                x,
                y,
            };
            match claims.iter_mut().find(|(name, _)| *name == label) {
                Some((_, seen)) => seen.push(mark),
                None => claims.push((label, vec![mark])),
            }
        }
    }
    assert!(
        !claims.is_empty(),
        "no registration marks were drawn at all"
    );

    // Every label, wherever it appears, names one model point.
    for (label, seen) in &claims {
        let first = seen[0];
        for mark in &seen[1..] {
            let drift = ((mark.x - first.x).powi(2) + (mark.y - first.y).powi(2)).sqrt();
            assert!(
                drift < 1e-3,
                "sheet {} puts {label} at ({}, {}), but sheet {} puts it at ({}, {}) — \
                 taping them by the labels would misalign the piece by {drift:.2}mm",
                mark.sheet,
                mark.x,
                mark.y,
                first.sheet,
                first.x,
                first.y
            );
        }
    }

    // And the marks have to actually be shared, or there is nothing to align
    // even though every label is self-consistent.
    let shared = claims.iter().filter(|(_, seen)| seen.len() > 1).count();
    assert!(
        shared >= 2,
        "only {shared} label(s) appear on more than one sheet; a shared edge \
         carries a whole column or row of crosses"
    );

    // The labelled points must be the crosses, not floating text: each one
    // needs a horizontal arm drawn through it on the sheet that names it.
    for (index, sheet) in sheets.iter().enumerate() {
        let (wx, wy) = sheet.window_origin_mm().unwrap();
        let (ox, oy) = sheet.tile_origin_mm().unwrap();
        for (label, x, y) in labelled_marks(sheet) {
            let has_arm = sheet.polylines.iter().filter(|p| p.len() == 2).any(|arm| {
                let horizontal = (arm[0].1.get() - arm[1].1.get()).abs() < 1e-6;
                if !horizontal {
                    return false;
                }
                let mid_x =
                    (arm[0].0.get() + arm[1].0.get()) / 2.0 / PT_PER_MM - wx.get() + ox.get();
                let arm_y = arm[0].1.get() / PT_PER_MM - wy.get() + oy.get();
                (mid_x - x).abs() < 1e-3 && (arm_y - y).abs() < 1e-3
            });
            assert!(
                has_arm,
                "sheet {index} draws the label {label} at ({x}, {y}) with no cross there"
            );
        }
    }
}

#[test]
fn sheet_numbering_runs_top_left_to_bottom_right() {
    // The order pages come out of the printer has to be the order they get
    // taped, or the operator is sorting sixteen identical-looking sheets.
    let piece = rect("Big Panel", 600.0, 700.0);
    let pdf = export_tiled_pdf(&project_of(&[&piece]), &PageLayout::a4()).expect("exports");
    let pages = parse(&pdf);

    let rows: usize = pages[1].tile_field("rows").unwrap().parse().unwrap();
    // The first sheet is the top row (highest row index), leftmost column.
    assert_eq!(pages[1].tile_field("col").as_deref(), Some("0"));
    assert_eq!(
        pages[1].tile_field("row").as_deref(),
        Some((rows - 1).to_string().as_str())
    );
    assert_eq!(pages[1].tile_piece().as_deref(), Some("Big Panel"));
    assert!(pages[1]
        .texts
        .iter()
        .any(|t| t.contains("sheet 1 of") && t.contains("Big Panel")));

    // And the last sheet is the bottom row, rightmost column.
    let last = pages.last().unwrap();
    assert_eq!(last.tile_field("row").as_deref(), Some("0"));
    let columns: usize = last.tile_field("columns").unwrap().parse().unwrap();
    assert_eq!(
        last.tile_field("col").as_deref(),
        Some((columns - 1).to_string().as_str())
    );
}

#[test]
fn each_piece_gets_its_own_grid_at_its_own_origin() {
    let front = rect("Front", 100.0, 100.0);
    let back = rect("Back", 400.0, 100.0);
    let pdf = export_tiled_pdf(&project_of(&[&front, &back]), &PageLayout::a4()).expect("exports");
    let pages = parse(&pdf);

    let names: Vec<String> = pages[1..]
        .iter()
        .map(|p| p.tile_piece().expect("every sheet names its piece"))
        .collect();
    assert_eq!(names[0], "Front");
    assert!(
        names[1..].iter().all(|n| n == "Back"),
        "pieces are not interleaved: {names:?}"
    );
    assert!(names.len() >= 4, "the wide piece needs several sheets");
}
