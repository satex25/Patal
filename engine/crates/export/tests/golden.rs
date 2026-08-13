//! The byte golden, and the reason it is only the second tier.
//!
//! A golden-file test on a PDF is a liability if it is the *only* test. It
//! goes red for reasons nobody can act on, and a check that fails for reasons
//! you cannot act on is one people learn to re-bless without reading — at
//! which point it protects nothing and costs a commit every few weeks.
//!
//! Two things make this one worth keeping. First, the writer is
//! dependency-free and stamps no clock, so nothing outside this crate can
//! move the bytes: a red golden here is always caused by the commit under
//! test. Second, the real assertions live elsewhere — `cut_line_is_the_kernels`
//! and `tiles_reassemble` check meaning, by parsing the document back. This
//! file only answers "did anything at all change", and when the answer is yes
//! it prints where, so the diff is read rather than re-blessed.
//!
//! To update deliberately, after reading the diff:
//!
//! ```text
//! PATAL_BLESS_GOLDEN=1 cargo test -p patal-export --test golden
//! ```

use std::path::PathBuf;

use patal_export::{export_tiled_pdf, PageLayout};
use patal_geometry::{PatternBoundary, Point2};
use patal_pattern::PatternPiece;

/// The fixture piece. Deliberately boring and deliberately fixed: a shape
/// that spans four sheets, so the golden covers tiling, registration marks,
/// the calibration strip and the calibration page in one file.
fn fixture_piece() -> PatternPiece {
    let mut piece = PatternPiece::new(
        "Golden Panel",
        PatternBoundary::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(300.0, 0.0),
            Point2::new(300.0, 250.0),
            Point2::new(120.0, 320.0),
            Point2::new(0.0, 250.0),
        ])
        .expect("a valid boundary"),
    );
    piece
        .set_seam_allowance_mm(12.0)
        .expect("12mm is a valid allowance");
    piece
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("golden-panel-a4.pdf")
}

#[test]
fn the_golden_panel_exports_to_exactly_the_bytes_it_did_last_time() {
    let pdf = export_tiled_pdf(&[&fixture_piece()], &PageLayout::a4()).expect("exports");
    let path = fixture_path();

    if std::env::var_os("PATAL_BLESS_GOLDEN").is_some() {
        std::fs::create_dir_all(path.parent().expect("fixtures directory"))
            .expect("can create the fixtures directory");
        std::fs::write(&path, &pdf).expect("can write the golden file");
        eprintln!("blessed {} ({} bytes)", path.display(), pdf.len());
        return;
    }

    let expected = std::fs::read(&path).unwrap_or_else(|err| {
        panic!(
            "cannot read the golden file at {}: {err}. \
             Create it with PATAL_BLESS_GOLDEN=1.",
            path.display()
        )
    });

    if pdf == expected {
        return;
    }

    // The readable diff. Without this, a failure is "assertion failed:
    // left == right" over two hundred-kilobyte byte arrays, which is how a
    // golden test gets re-blessed unread.
    let actual_path = std::env::temp_dir().join("patal-golden-actual.pdf");
    let _ = std::fs::write(&actual_path, &pdf);

    let at = pdf
        .iter()
        .zip(&expected)
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| pdf.len().min(expected.len()));

    let context = |bytes: &[u8]| -> String {
        let from = at.saturating_sub(60);
        let to = (at + 60).min(bytes.len());
        String::from_utf8_lossy(&bytes[from..to])
            .replace('\n', "\\n")
            .to_string()
    };

    panic!(
        "the exported PDF no longer matches the golden file.\n\
         \n  golden : {} bytes\n  actual : {} bytes\n  first difference at byte {at}\n\
         \n  golden : ...{}...\n  actual : ...{}...\n\
         \nThe full output was written to {}.\n\
         If the change is intended, read the difference above, then re-run with \
         PATAL_BLESS_GOLDEN=1.",
        expected.len(),
        pdf.len(),
        context(&expected),
        context(&pdf),
        actual_path.display(),
    );
}

#[test]
fn the_golden_file_is_a_pdf_that_says_what_it_is() {
    // A cheap guard against a blessed file that is empty, truncated, or the
    // output of a half-finished refactor. Byte equality alone would happily
    // hold two identical broken files equal forever.
    let bytes = std::fs::read(fixture_path()).expect("the golden file exists");
    assert!(bytes.starts_with(b"%PDF-1.7\n"));
    assert!(bytes.ends_with(b"%%EOF\n"));

    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("/Count 5"),
        "calibration page plus a 2x2 grid"
    );
    assert!(text.contains("patal-calibration-page"));
    assert!(text.contains("50mm calibration square"));
    assert!(!text.contains("/CreationDate"));
    assert!(
        text.contains("/MediaBox [0 0 595.2756 841.8898]"),
        "A4 at true scale"
    );
}
