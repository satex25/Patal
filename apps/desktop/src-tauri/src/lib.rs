use patruin_geometry::{PatternBoundary, Point2};
use patruin_pattern::{PatternPiece, Project};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Proves the desktop app talks to the same engine crates as everything
/// else — this is a placeholder call until the UI builds real projects.
#[tauri::command]
fn engine_demo_perimeter_mm() -> f64 {
    let boundary = PatternBoundary::new(vec![
        Point2::new(0.0, 0.0),
        Point2::new(200.0, 0.0),
        Point2::new(200.0, 300.0),
        Point2::new(0.0, 300.0),
    ]);
    let mut project = Project::new("Demo Project");
    project.add_piece(PatternPiece::new("Demo Panel", boundary));
    project.total_perimeter_mm()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, engine_demo_perimeter_mm])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
