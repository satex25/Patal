//! Pātāl's engineering harness.
//!
//! **This is not the product.** ADR-001 rejected Tauri as a *shipping*
//! target on native-feel grounds and that decision stands. ADR-005 records
//! why this app is nonetheless unfrozen: it is the only thing in the repo
//! that runs on the machine Pātāl is developed on, and it links the engine
//! crates directly with no FFI in between. That makes it the cheapest
//! possible way to look at geometry with your eyes instead of reading it out
//! of a test assertion.
//!
//! Everything here is disposable. It exists to answer questions —
//! *does that neckline actually look right at 0.4mm?*, *does a `.patal` file
//! survive a round trip through a real disk?* — and it should be deleted
//! without ceremony when the native app can answer them instead.
//!
//! Because it is a harness rather than a product, it is allowed to do things
//! the engine is not: read and write files directly, hardcode a demo piece,
//! and report timings in the UI.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use patal_export::{export_tiled_pdf, PageLayout};
use patal_geometry::{EdgeSegment, PatternBoundary, Point2, SeamPath};
use patal_materials::Material;
use patal_pattern::{Document, PatternPiece, Project};
use serde::Serialize;

/// A bodice front with a scooped neckline and a shaped armhole — four cubics
/// and two straight seams. The same shape the drag-loop benchmark uses, so
/// what you see here is what was measured there.
fn bodice_front() -> Result<SeamPath, String> {
    let start = Point2::new(0.0, 0.0);
    SeamPath::closed(
        start,
        vec![
            EdgeSegment::Cubic {
                c1: Point2::new(15.0, -30.0),
                c2: Point2::new(50.0, -22.0),
                to: Point2::new(75.0, 10.0),
            },
            EdgeSegment::Line {
                to: Point2::new(140.0, 30.0),
            },
            EdgeSegment::Cubic {
                c1: Point2::new(175.0, -30.0),
                c2: Point2::new(140.0, -130.0),
                to: Point2::new(160.0, -200.0),
            },
            EdgeSegment::Cubic {
                c1: Point2::new(140.0, -280.0),
                c2: Point2::new(165.0, -350.0),
                to: Point2::new(150.0, -420.0),
            },
            EdgeSegment::Cubic {
                c1: Point2::new(100.0, -430.0),
                c2: Point2::new(40.0, -425.0),
                to: Point2::new(0.0, -420.0),
            },
            EdgeSegment::Line { to: start },
        ],
    )
    .map_err(|err| err.to_string())
}

fn as_pairs(boundary: &PatternBoundary) -> Vec<[f64; 2]> {
    boundary.points().iter().map(|p| [p.x, p.y]).collect()
}

/// What the canvas needs to draw one frame of the preview.
#[derive(Serialize)]
struct CutPreview {
    /// The authored path, flattened — the sewing line.
    outline: Vec<[f64; 2]>,
    /// The cut line, or `None` when the allowance cannot be applied.
    cut: Option<Vec<[f64; 2]>>,
    /// Why the cut line is missing, in the engine's own words.
    ///
    /// Shown rather than swallowed. Watching a real error appear the moment
    /// an allowance exceeds what a curve can give is the entire reason to
    /// have a visual harness — that is the message a designer will
    /// eventually see, and this is where it gets read by a human first.
    error: Option<String>,
    outline_vertices: usize,
    /// Wall time for flatten + offset + the self-intersection check, which
    /// is what a drag would pay per frame.
    elapsed_micros: u128,
}

/// Flatten the piece at `tolerance_mm`, then offset it by `allowance_mm`.
///
/// A failing offset is a successful command: the preview reports the error
/// alongside the outline that produced it, so the shape stays on screen
/// while you work out why the allowance will not fit.
#[tauri::command]
fn cut_preview(tolerance_mm: f64, allowance_mm: f64) -> Result<CutPreview, String> {
    let path = bodice_front()?;

    let started = Instant::now();
    let outline = path
        .flatten_for_offset(tolerance_mm, allowance_mm)
        .map_err(|err| err.to_string())?;
    let cut = outline.offset(allowance_mm);
    let elapsed_micros = started.elapsed().as_micros();

    let (cut, error) = match cut {
        Ok(boundary) => (Some(as_pairs(&boundary)), None),
        Err(err) => (None, Some(err.to_string())),
    };

    Ok(CutPreview {
        outline_vertices: outline.points().len(),
        outline: as_pairs(&outline),
        cut,
        error,
        elapsed_micros,
    })
}

#[derive(Serialize)]
struct SaveReport {
    path: String,
    bytes: usize,
    schema_version: u32,
    /// Whether the document read back from disk equals what was written.
    round_tripped: bool,
    /// What the reloaded piece resolves its material to, proving the
    /// reference survived the file rather than being re-embedded.
    material_name: Option<String>,
}

/// Write a real `.patal` file to disk and read it back.
///
/// The engine has no persistence layer — that was deliberately cut from this
/// wave — so the file handling lives here, in the disposable app, where it
/// exercises the document format without committing the engine to an API for
/// it. What this proves is narrow and worth proving: a document written to a
/// real file comes back as the same document, with its material references
/// intact.
#[tauri::command]
fn save_demo_document(directory: String, tolerance_mm: f64) -> Result<SaveReport, String> {
    let path = bodice_front()?;
    let boundary = path.flatten(tolerance_mm).map_err(|err| err.to_string())?;

    let mut project = Project::new("Harness Demo");
    let wool = project.materials.add(Material::new("Wool Crepe"));

    let mut piece = PatternPiece::new("Bodice Front", boundary);
    piece.material = Some(wool);
    project.add_piece(piece);

    let document = Document::new(project);
    let json = serde_json::to_string_pretty(&document).map_err(|err| err.to_string())?;

    let mut file: PathBuf = PathBuf::from(&directory);
    file.push("harness-demo.patal");
    fs::write(&file, &json).map_err(|err| err.to_string())?;

    let reloaded: Document =
        serde_json::from_str(&fs::read_to_string(&file).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let reloaded_piece = reloaded
        .project
        .find_piece("Bodice Front")
        .ok_or("the reloaded document lost its piece")?;
    let material_name = reloaded
        .project
        .material_for(reloaded_piece)
        .map_err(|err| err.to_string())?
        .map(|m| m.name.clone());

    let round_tripped =
        serde_json::to_string_pretty(&reloaded).map_err(|err| err.to_string())? == json;

    Ok(SaveReport {
        path: file.display().to_string(),
        bytes: json.len(),
        schema_version: reloaded.schema_version(),
        round_tripped,
        material_name,
    })
}

#[derive(Serialize)]
struct ExportReport {
    path: String,
    bytes: usize,
    pages: usize,
    /// The page size the PDF declares, so the operator can set the printer
    /// driver's paper size to match it. A Letter tray printing an A4
    /// document shrinks it by about six per cent and says nothing.
    page_size: String,
}

/// Write a tiled, true-scale PDF of the demo piece.
///
/// The harness owns the file, not the export crate. `patal-export` returns
/// bytes and touches no `std::fs`, which keeps the core free of the platform
/// (ADR-001) and, more usefully, makes the failure mode benign: an I/O error
/// here cannot leave a truncated PDF behind, because every byte was produced
/// before the first one was written.
///
/// Even so the write goes through a temporary file and a rename. A rename
/// within a directory is atomic on both platforms this runs on, so what sits
/// at the destination is either the previous complete file or the new
/// complete file, never four pages of a five-page pattern. A half-written PDF
/// is the sharpest version of the defect "correct or loud" exists to forbid:
/// it opens, it prints, and nothing about the paper says it is incomplete.
#[tauri::command]
fn export_demo_pdf(
    directory: String,
    tolerance_mm: f64,
    letter: bool,
) -> Result<ExportReport, String> {
    let path = bodice_front()?;
    let boundary = path.flatten(tolerance_mm).map_err(|err| err.to_string())?;
    let piece = PatternPiece::new("Bodice Front", boundary);

    let layout = if letter {
        PageLayout::letter()
    } else {
        PageLayout::a4()
    };
    let pdf = export_tiled_pdf(&[&piece], &layout).map_err(|err| err.to_string())?;

    let mut file: PathBuf = PathBuf::from(&directory);
    file.push("harness-demo.pdf");
    let mut temporary = file.clone();
    temporary.set_extension("pdf.writing");

    fs::write(&temporary, &pdf).map_err(|err| err.to_string())?;
    fs::rename(&temporary, &file).map_err(|err| {
        // Clean up rather than leave a stray `.pdf.writing` beside the real
        // file for someone to find later and wonder about.
        let _ = fs::remove_file(&temporary);
        err.to_string()
    })?;

    Ok(ExportReport {
        pages: String::from_utf8_lossy(&pdf)
            .matches("/Type /Page ")
            .count(),
        path: file.display().to_string(),
        bytes: pdf.len(),
        page_size: format!(
            "{} x {} mm",
            layout.page_width().get(),
            layout.page_height().get()
        ),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            cut_preview,
            save_demo_document,
            export_demo_pdf
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_preview_produces_a_drawable_cut_line() {
        let preview = cut_preview(0.4, 10.0).expect("the demo piece flattens");
        assert!(preview.outline_vertices >= 3);
        assert_eq!(preview.outline.len(), preview.outline_vertices);
        assert!(preview.error.is_none(), "{:?}", preview.error);
        assert!(preview.cut.is_some());
    }

    #[test]
    fn a_tighter_tolerance_buys_more_vertices() {
        let coarse = cut_preview(1.0, 10.0).unwrap();
        let fine = cut_preview(0.01, 10.0).unwrap();
        assert!(fine.outline_vertices > coarse.outline_vertices);
    }

    #[test]
    fn an_impossible_allowance_reports_instead_of_drawing_nonsense() {
        // Far past what the neckline's curvature can give. The harness must
        // surface the engine's refusal, not fail the command and blank the
        // screen — seeing the error next to the shape that caused it is the
        // reason to have a window at all.
        let preview = cut_preview(0.4, -200.0).expect("the command still succeeds");
        assert!(preview.cut.is_none());
        assert!(preview.error.is_some());
    }

    #[test]
    fn a_document_survives_a_real_file() {
        let directory = std::env::temp_dir();
        let report = save_demo_document(directory.display().to_string(), 0.4)
            .expect("writes and reads back");

        assert!(report.bytes > 0);
        assert_eq!(report.schema_version, patal_pattern::SCHEMA_VERSION);
        assert!(
            report.round_tripped,
            "a .patal file must reload identically"
        );
        assert_eq!(report.material_name.as_deref(), Some("Wool Crepe"));

        let _ = fs::remove_file(directory.join("harness-demo.patal"));
    }

    #[test]
    fn the_demo_piece_exports_to_a_real_pdf_on_disk() {
        let directory = std::env::temp_dir();
        let report = export_demo_pdf(directory.display().to_string(), 0.4, false)
            .expect("the demo piece exports");

        assert!(
            report.pages >= 2,
            "a calibration page and at least one sheet"
        );
        assert_eq!(report.page_size, "210 x 297 mm");

        let written = fs::read(&report.path).expect("the file is there");
        assert_eq!(written.len(), report.bytes);
        assert!(written.starts_with(b"%PDF-1.7"));
        assert!(written.ends_with(b"%%EOF\n"), "not truncated");

        // The temporary must not survive a successful write.
        assert!(
            !directory.join("harness-demo.pdf.writing").exists(),
            "the in-progress file was left behind"
        );

        let _ = fs::remove_file(&report.path);
    }

    #[test]
    fn exporting_twice_replaces_the_file_rather_than_appending_to_it() {
        // The rename path, exercised against an existing destination —
        // the case where a non-atomic write would leave the tail of the
        // previous, longer document glued to the end of the new one.
        let directory = std::env::temp_dir().join("patal-export-twice");
        fs::create_dir_all(&directory).expect("can create a scratch directory");

        let first = export_demo_pdf(directory.display().to_string(), 1.0, false).unwrap();
        let second = export_demo_pdf(directory.display().to_string(), 0.1, false).unwrap();

        let written = fs::read(&second.path).expect("the file is there");
        assert_eq!(written.len(), second.bytes);
        assert_ne!(
            first.bytes, second.bytes,
            "a finer tolerance should produce a different document"
        );
        assert_eq!(
            String::from_utf8_lossy(&written).matches("%%EOF").count(),
            1,
            "two end-of-file markers means the previous document's tail is \
             still glued to the end of this one"
        );

        let _ = fs::remove_dir_all(&directory);
    }
}
