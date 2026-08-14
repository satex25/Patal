//! Page geometry, validated at construction, and the tile grid it implies.
//!
//! Every number a page needs is checked once, in [`PageLayout::new`], and is
//! a guaranteed-sane invariant everywhere after that. The alternative —
//! validating at use — is how `overlap >= usable` becomes a tiling loop that
//! never terminates rather than an error somebody can read.

use crate::error::ExportError;
use crate::units::Mm;

/// The height reserved at the foot of every content page for the calibration
/// square and its label.
///
/// It is reserved rather than overlaid. A 50mm box drawn on top of a tiled
/// piece will, on some sheet of some piece, land across the cut line — and a
/// square someone has been told to measure, sitting on top of a line someone
/// is about to cut, is a genuine hazard rather than a cosmetic one. Reserving
/// the strip costs paper and makes the collision structurally impossible.
pub const CALIBRATION_STRIP_MM: f64 = 58.0;

/// The side of the calibration square. Chosen because it is the number every
/// pattern-printing instruction in the world already quotes.
pub const CALIBRATION_SQUARE_MM: f64 = 50.0;

/// The smallest margin this crate will accept.
///
/// Consumer inkjets hold roughly 3-6mm of physically unprintable edge. Below
/// this the cut line does not get clipped visibly, it simply does not come
/// out of the printer, and the piece loses a strip nobody sees go missing.
pub const MIN_MARGIN_MM: f64 = 6.0;

/// The default, comfortably clear of every consumer printer's dead band.
pub const DEFAULT_MARGIN_MM: f64 = 10.0;

/// Enough to align two sheets by eye without wasting a third of the paper.
pub const DEFAULT_OVERLAP_MM: f64 = 10.0;

/// A 20x20 grid. On A4 that is about 3.7m by 4.4m of taped paper — past any
/// real garment piece, and comfortably short of the print job a piece
/// accidentally authored in metres would produce.
pub const MAX_TILES_PER_PIECE: usize = 400;

/// The sheet, and how sheets relate to their neighbours.
///
/// Note what is *not* here: a scale factor. There is no way to ask this crate
/// for a half-size print. The one job of a pattern exporter is that a
/// millimetre in the model is a millimetre on the paper, and a `scale`
/// parameter is how that guarantee becomes conditional on an argument nobody
/// checks at the printer. If a preview mode is ever genuinely needed it has
/// to stamp `NOT TO SCALE` on every page and in the filename, and it can be
/// added then, loudly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageLayout {
    page_width: Mm,
    page_height: Mm,
    margin: Mm,
    overlap: Mm,
}

impl PageLayout {
    /// ISO A4, 210 x 297 mm.
    pub fn a4() -> Self {
        Self::new(
            Mm(210.0),
            Mm(297.0),
            Mm(DEFAULT_MARGIN_MM),
            Mm(DEFAULT_OVERLAP_MM),
        )
        .expect("A4 with default margins is a valid layout")
    }

    /// US Letter, 8.5 x 11 in, expressed in millimetres because the rest of
    /// the crate has exactly one unit.
    pub fn letter() -> Self {
        Self::new(
            Mm(215.9),
            Mm(279.4),
            Mm(DEFAULT_MARGIN_MM),
            Mm(DEFAULT_OVERLAP_MM),
        )
        .expect("Letter with default margins is a valid layout")
    }

    /// Validates a page description, or explains what is wrong with it.
    pub fn new(
        page_width: Mm,
        page_height: Mm,
        margin: Mm,
        overlap: Mm,
    ) -> Result<Self, ExportError> {
        for (field, value, must_be_positive) in [
            ("page_width_mm", page_width.get(), true),
            ("page_height_mm", page_height.get(), true),
            ("margin_mm", margin.get(), false),
            ("overlap_mm", overlap.get(), false),
        ] {
            if !value.is_finite() || value < 0.0 || (must_be_positive && value <= 0.0) {
                return Err(ExportError::InvalidPageGeometry { field, value });
            }
        }

        if margin.get() < MIN_MARGIN_MM {
            return Err(ExportError::MarginBelowPrintableArea {
                margin_mm: margin.get(),
                minimum_mm: MIN_MARGIN_MM,
            });
        }

        let layout = Self {
            page_width,
            page_height,
            margin,
            overlap,
        };

        // The horizontal window loses two margins; the vertical one loses two
        // margins and the calibration strip.
        let needed_width = margin.get() * 2.0;
        if layout.window_width().get() <= 0.0 {
            return Err(ExportError::PageTooSmall {
                page_mm: page_width.get(),
                needed_mm: needed_width,
                axis: "width",
            });
        }
        let needed_height = margin.get() * 2.0 + CALIBRATION_STRIP_MM;
        if layout.window_height().get() <= 0.0 {
            return Err(ExportError::PageTooSmall {
                page_mm: page_height.get(),
                needed_mm: needed_height,
                axis: "height",
            });
        }

        // The termination condition. `advance = window - overlap`, and a
        // non-positive advance is a loop that never finishes rather than a
        // wrong answer, so it is rejected here where the numbers are still
        // together.
        for (axis, window) in [
            ("width", layout.window_width()),
            ("height", layout.window_height()),
        ] {
            if overlap.get() >= window.get() {
                return Err(ExportError::OverlapExceedsUsableArea {
                    overlap_mm: overlap.get(),
                    usable_mm: window.get(),
                    axis,
                });
            }
        }

        Ok(layout)
    }

    pub fn page_width(self) -> Mm {
        self.page_width
    }

    pub fn page_height(self) -> Mm {
        self.page_height
    }

    pub fn margin(self) -> Mm {
        self.margin
    }

    pub fn overlap(self) -> Mm {
        self.overlap
    }

    /// The drawable width of one sheet.
    pub fn window_width(self) -> Mm {
        self.page_width - self.margin * 2.0
    }

    /// The drawable height of one sheet, after the calibration strip.
    pub fn window_height(self) -> Mm {
        self.page_height - self.margin * 2.0 - Mm(CALIBRATION_STRIP_MM)
    }

    /// How far the model advances from one sheet to the next. Guaranteed
    /// positive by [`PageLayout::new`].
    pub fn advance_x(self) -> Mm {
        self.window_width() - self.overlap
    }

    pub fn advance_y(self) -> Mm {
        self.window_height() - self.overlap
    }

    /// Page-space coordinate of the drawable window's bottom-left corner.
    pub fn window_origin(self) -> (Mm, Mm) {
        (self.margin, self.margin + Mm(CALIBRATION_STRIP_MM))
    }
}

/// An axis-aligned box in model millimetres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundsMm {
    pub min_x: Mm,
    pub min_y: Mm,
    pub max_x: Mm,
    pub max_y: Mm,
}

impl BoundsMm {
    /// The bounding box of a non-empty point set.
    ///
    /// Takes a slice that a caller has already established is non-empty —
    /// every path into this function goes through a `PatternBoundary`, which
    /// the geometry crate guarantees holds at least three finite points.
    pub fn of(points: &[patal_geometry::Point2]) -> Option<Self> {
        let first = points.first()?;
        let mut bounds = Self {
            min_x: Mm(first.x),
            min_y: Mm(first.y),
            max_x: Mm(first.x),
            max_y: Mm(first.y),
        };
        for p in points {
            bounds.min_x = bounds.min_x.min(Mm(p.x));
            bounds.min_y = bounds.min_y.min(Mm(p.y));
            bounds.max_x = bounds.max_x.max(Mm(p.x));
            bounds.max_y = bounds.max_y.max(Mm(p.y));
        }
        Some(bounds)
    }

    /// The bounding box of a boundary.
    ///
    /// Infallible because `PatternBoundary::new` refuses anything with fewer
    /// than three finite points, so there is no empty case to represent. The
    /// `expect` is the loud form of that invariant: if the geometry crate
    /// ever hands out an empty boundary, this crate stops rather than
    /// inventing a zero-sized page.
    pub fn of_boundary(boundary: &patal_geometry::PatternBoundary) -> Self {
        Self::of(boundary.points())
            .expect("PatternBoundary guarantees at least three finite points")
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }

    pub fn width(self) -> Mm {
        self.max_x - self.min_x
    }

    pub fn height(self) -> Mm {
        self.max_y - self.min_y
    }
}

/// How many sheets a piece takes, and where each one starts in model space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileGrid {
    pub bounds: BoundsMm,
    pub columns: usize,
    pub rows: usize,
    pub advance_x: Mm,
    pub advance_y: Mm,
    pub window_width: Mm,
    pub window_height: Mm,
}

impl TileGrid {
    /// Covers `bounds` with overlapping windows.
    ///
    /// `n` windows of width `W` advancing by `A` reach `A·(n−1) + W`, so the
    /// count is `ceil((extent − W) / A) + 1`, floored at one. The `+ 1e-9`
    /// absorbs the case where the extent is an exact multiple of the advance
    /// and binary floating point lands a hair over it — without it, a piece
    /// exactly 190mm wide on a 190mm window picks up a second, empty sheet.
    pub fn cover(bounds: BoundsMm, layout: PageLayout) -> Self {
        let count = |extent: Mm, window: Mm, advance: Mm| -> usize {
            if extent.get() <= window.get() + 1e-9 {
                return 1;
            }
            (((extent.get() - window.get()) / advance.get()) - 1e-9).ceil() as usize + 1
        };

        Self {
            bounds,
            columns: count(bounds.width(), layout.window_width(), layout.advance_x()),
            rows: count(bounds.height(), layout.window_height(), layout.advance_y()),
            advance_x: layout.advance_x(),
            advance_y: layout.advance_y(),
            window_width: layout.window_width(),
            window_height: layout.window_height(),
        }
    }

    pub fn tiles(self) -> usize {
        self.columns * self.rows
    }

    /// The model coordinate that lands on the bottom-left of the drawable
    /// window for tile `(column, row)`.
    pub fn tile_origin(self, column: usize, row: usize) -> (Mm, Mm) {
        (
            self.bounds.min_x + self.advance_x * column as f64,
            self.bounds.min_y + self.advance_y * row as f64,
        )
    }

    /// The model x coordinates of every shared grid line, including both
    /// outer edges.
    ///
    /// These are what the registration crosses are drawn on. A grid line at
    /// `min_x + k·advance` falls inside the window of tile `k−1` (near its
    /// right edge) *and* tile `k` (at its left edge), so the same mark
    /// appears on both sheets at the same model coordinate. Lining the two
    /// crosses up is therefore the same thing as assembling the piece
    /// correctly — the marks are not decoration, they are the contract.
    pub fn grid_x(self) -> Vec<Mm> {
        (0..=self.columns)
            .map(|k| self.bounds.min_x + self.advance_x * k as f64)
            .collect()
    }

    pub fn grid_y(self) -> Vec<Mm> {
        (0..=self.rows)
            .map(|k| self.bounds.min_y + self.advance_y * k as f64)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_windows_are_what_the_arithmetic_says() {
        let a4 = PageLayout::a4();
        assert_eq!(a4.window_width(), Mm(210.0 - 20.0));
        assert!((a4.window_height().get() - (297.0 - 20.0 - 58.0)).abs() < 1e-12);
        assert!((a4.advance_x().get() - 180.0).abs() < 1e-12);
    }

    #[test]
    fn letter_is_also_valid_with_the_defaults() {
        let letter = PageLayout::letter();
        assert!(letter.window_width().get() > 0.0);
        assert!(letter.advance_y().get() > 0.0);
    }

    #[test]
    fn a_non_finite_page_is_refused_by_name() {
        // Compared by hand rather than with `assert_eq!`: NaN is not equal to
        // itself, so the derived `PartialEq` on the error would report a
        // mismatch between two identical values.
        match PageLayout::new(Mm(f64::NAN), Mm(297.0), Mm(10.0), Mm(10.0)) {
            Err(ExportError::InvalidPageGeometry { field, value }) => {
                assert_eq!(field, "page_width_mm");
                assert!(value.is_nan());
            }
            other => panic!("expected a named page-geometry error, got {other:?}"),
        }
    }

    #[test]
    fn a_zero_height_page_is_refused_by_name() {
        assert_eq!(
            PageLayout::new(Mm(210.0), Mm(0.0), Mm(10.0), Mm(10.0)),
            Err(ExportError::InvalidPageGeometry {
                field: "page_height_mm",
                value: 0.0,
            })
        );
    }

    #[test]
    fn a_negative_overlap_is_refused_by_name() {
        assert_eq!(
            PageLayout::new(Mm(210.0), Mm(297.0), Mm(10.0), Mm(-1.0)),
            Err(ExportError::InvalidPageGeometry {
                field: "overlap_mm",
                value: -1.0,
            })
        );
    }

    #[test]
    fn a_thin_margin_is_refused_rather_than_silently_clipped() {
        assert_eq!(
            PageLayout::new(Mm(210.0), Mm(297.0), Mm(2.0), Mm(10.0)),
            Err(ExportError::MarginBelowPrintableArea {
                margin_mm: 2.0,
                minimum_mm: MIN_MARGIN_MM,
            })
        );
    }

    #[test]
    fn an_overlap_that_would_never_advance_is_refused() {
        // The hang, caught as an error. A 190mm window with a 190mm overlap
        // advances zero millimetres per sheet.
        let err = PageLayout::new(Mm(210.0), Mm(297.0), Mm(10.0), Mm(190.0)).unwrap_err();
        assert!(
            matches!(err, ExportError::OverlapExceedsUsableArea { .. }),
            "{err:?}"
        );
        assert!(err.to_string().contains("would not advance"), "{err}");
    }

    #[test]
    fn a_page_with_no_room_left_after_the_strip_is_refused() {
        // 60mm tall: two 10mm margins and a 58mm calibration strip need 78.
        let err = PageLayout::new(Mm(210.0), Mm(60.0), Mm(10.0), Mm(5.0)).unwrap_err();
        assert!(matches!(
            err,
            ExportError::PageTooSmall { axis: "height", .. }
        ));
    }

    fn bounds(w: f64, h: f64) -> BoundsMm {
        BoundsMm {
            min_x: Mm(0.0),
            min_y: Mm(0.0),
            max_x: Mm(w),
            max_y: Mm(h),
        }
    }

    #[test]
    fn a_piece_that_fits_takes_exactly_one_sheet() {
        let grid = TileGrid::cover(bounds(100.0, 100.0), PageLayout::a4());
        assert_eq!((grid.columns, grid.rows), (1, 1));
    }

    #[test]
    fn a_piece_exactly_the_window_wide_does_not_pick_up_an_empty_sheet() {
        let a4 = PageLayout::a4();
        let grid = TileGrid::cover(bounds(a4.window_width().get(), 100.0), a4);
        assert_eq!(grid.columns, 1);
    }

    #[test]
    fn tiles_cover_the_bounds_with_the_stated_overlap() {
        let a4 = PageLayout::a4();
        let b = bounds(700.0, 900.0);
        let grid = TileGrid::cover(b, a4);

        // Coverage: the last window's far edge must reach the far edge of the
        // piece. This is the assertion that fails when the count is off by
        // one and a strip of the piece is simply never printed.
        let reach_x = grid.tile_origin(grid.columns - 1, 0).0 + grid.window_width;
        assert!(reach_x.get() >= b.max_x.get() - 1e-9, "{reach_x:?}");
        let reach_y = grid.tile_origin(0, grid.rows - 1).1 + grid.window_height;
        assert!(reach_y.get() >= b.max_y.get() - 1e-9, "{reach_y:?}");

        // Overlap: consecutive windows share exactly `overlap` millimetres.
        for c in 1..grid.columns {
            let previous_far = grid.tile_origin(c - 1, 0).0 + grid.window_width;
            let shared = previous_far - grid.tile_origin(c, 0).0;
            assert!(
                (shared.get() - a4.overlap().get()).abs() < 1e-9,
                "column {c} shares {shared:?}"
            );
        }
    }

    #[test]
    fn a_grid_line_lands_inside_both_sheets_that_share_it() {
        let a4 = PageLayout::a4();
        let grid = TileGrid::cover(bounds(700.0, 200.0), a4);
        assert!(grid.columns >= 3);

        let line = grid.grid_x()[1];
        let left = grid.tile_origin(0, 0).0;
        let right = grid.tile_origin(1, 0).0;
        assert!(line.get() >= left.get() && line.get() <= (left + grid.window_width).get());
        assert!(line.get() >= right.get() && line.get() <= (right + grid.window_width).get());
    }
}
