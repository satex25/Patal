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

        let _ = fs::remove_file(PathBuf::from(directory).join("harness-demo.patal"));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![cut_preview, save_demo_document])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
