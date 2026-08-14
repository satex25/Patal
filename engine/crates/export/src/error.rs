//! Every way this crate can refuse, named.
//!
//! `geometry/src/lib.rs` states the rule the whole engine is built on:
//! correct or loud. Never return a plausible-looking number from a fallible
//! operation. Export is the first crate where the plausible-looking result is
//! not a number but a *document* — one that opens, renders, prints, and is
//! wrong. A six-piece PDF emitted because the seventh piece failed is the
//! purest form of that defect: nothing about the artifact says it is
//! incomplete, and the person who finds out is holding scissors.
//!
//! So there is no partial success in this crate. Either every piece is laid
//! out and the bytes are returned, or nothing is.

use std::fmt;

use patal_pattern::PatternError;

/// Why an export did not happen.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportError {
    /// Nothing was passed in.
    ///
    /// A zero-page PDF is a valid file that no reader will open and no
    /// printer will print, so producing one turns a caller's mistake into a
    /// mystery twenty minutes downstream.
    NothingToExport,

    /// A page dimension, margin, or overlap that cannot describe paper.
    ///
    /// `field` names which one, because "invalid page geometry" against four
    /// candidate numbers is a message that sends someone reading source.
    InvalidPageGeometry { field: &'static str, value: f64 },

    /// The margin is inside the band most consumer printers physically
    /// cannot reach.
    ///
    /// Inkjets typically hold 3-6mm of unprintable edge, and a cut line that
    /// falls in it is not clipped visibly — it simply is not there, and the
    /// piece silently loses a strip. This is the loud form of the warning
    /// the plan asked for: the crate has no log channel to warn through, and
    /// a warning nobody sees is the same as no warning.
    MarginBelowPrintableArea { margin_mm: f64, minimum_mm: f64 },

    /// After margins and the calibration strip, no drawable area is left.
    PageTooSmall {
        page_mm: f64,
        needed_mm: f64,
        axis: &'static str,
    },

    /// The overlap eats the whole drawable window, so consecutive tiles make
    /// no forward progress.
    ///
    /// Worth its own variant because the naive implementation of this case is
    /// not a wrong answer, it is a loop that never terminates. An export that
    /// hangs is indistinguishable from one that is slow, and the caller waits
    /// instead of reading an error.
    ///
    /// Named against the *usable window*, not the page: the page is not the
    /// constraint, the window left after margins and the calibration strip
    /// is, and reporting the page dimension here would point at the wrong
    /// number.
    OverlapExceedsUsableArea {
        overlap_mm: f64,
        usable_mm: f64,
        axis: &'static str,
    },

    /// A piece needs more sheets than anyone is going to tape together.
    ///
    /// The bound exists so that a piece whose coordinates are in metres
    /// because someone confused units produces an error rather than a
    /// six-hour print job.
    TooManyTiles {
        piece: String,
        tiles: usize,
        maximum: usize,
    },

    /// The kernel could not produce a cut line for this piece.
    ///
    /// Propagated with the piece named. The underlying
    /// [`PatternError`] already explains *what* went wrong — a seam allowance
    /// that exceeds a curve's radius, most often — but not *where*, and a
    /// project has more than one piece.
    CutLineFailed { piece: String, source: PatternError },
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NothingToExport => write!(
                f,
                "nothing to export: no pieces were given, and an empty PDF is \
                 not a useful answer"
            ),
            Self::InvalidPageGeometry { field, value } => write!(
                f,
                "page geometry is invalid: {field} is {value}, which must be a \
                 finite, positive number of millimetres"
            ),
            Self::MarginBelowPrintableArea {
                margin_mm,
                minimum_mm,
            } => write!(
                f,
                "a {margin_mm}mm margin is inside the edge most printers cannot \
                 reach; use at least {minimum_mm}mm, or the cut line loses a \
                 strip without saying so"
            ),
            Self::PageTooSmall {
                page_mm,
                needed_mm,
                axis,
            } => write!(
                f,
                "a {page_mm}mm page leaves no drawable {axis}: margins and the \
                 calibration strip need {needed_mm}mm before anything is drawn"
            ),
            Self::OverlapExceedsUsableArea {
                overlap_mm,
                usable_mm,
                axis,
            } => write!(
                f,
                "a {overlap_mm}mm overlap does not fit inside a {usable_mm}mm \
                 usable {axis}: consecutive sheets would not advance"
            ),
            Self::TooManyTiles {
                piece,
                tiles,
                maximum,
            } => write!(
                f,
                "piece \"{piece}\" needs {tiles} sheets, past the {maximum}-sheet \
                 limit — check the piece's units before printing"
            ),
            Self::CutLineFailed { piece, source } => {
                write!(f, "piece \"{piece}\" has no cut line: {source}")
            }
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CutLineFailed { source, .. } => Some(source),
            Self::NothingToExport
            | Self::InvalidPageGeometry { .. }
            | Self::MarginBelowPrintableArea { .. }
            | Self::PageTooSmall { .. }
            | Self::OverlapExceedsUsableArea { .. }
            | Self::TooManyTiles { .. } => None,
        }
    }
}
