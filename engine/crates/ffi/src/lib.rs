//! The FFI boundary: the seam between the platform-agnostic Rust engine and
//! each native front end (Swift on iOS/iPad/Mac, the Tauri desktop app on
//! Windows).
//!
//! Domain crates (`patruin-geometry`, `patruin-materials`, `patruin-pattern`)
//! stay free of FFI concerns; this crate defines uniffi-friendly DTOs and
//! converts to/from the real domain types. The exported surface here is
//! intentionally small — it grows alongside the domain crates rather than
//! trying to expose everything up front.

uniffi::setup_scaffolding!();

/// An FFI-safe 2D point. Converts to/from `patruin_geometry::Point2`.
#[derive(uniffi::Record)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl From<Point> for patruin_geometry::Point2 {
    fn from(p: Point) -> Self {
        patruin_geometry::Point2::new(p.x, p.y)
    }
}

impl From<patruin_geometry::Point2> for Point {
    fn from(p: patruin_geometry::Point2) -> Self {
        Point { x: p.x, y: p.y }
    }
}

#[uniffi::export]
pub fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// The perimeter, in millimeters, of a closed polygon boundary.
#[uniffi::export]
pub fn boundary_perimeter(points: Vec<Point>) -> f64 {
    let boundary = patruin_geometry::PatternBoundary::new(
        points.into_iter().map(Into::into).collect(),
    );
    boundary.perimeter()
}

/// Offsets a closed polygon boundary outward by `distance_mm` — the seam
/// allowance construction, exposed across the FFI boundary.
#[uniffi::export]
pub fn offset_boundary(points: Vec<Point>, distance_mm: f64) -> Vec<Point> {
    let boundary = patruin_geometry::PatternBoundary::new(
        points.into_iter().map(Into::into).collect(),
    );
    boundary
        .offset(distance_mm)
        .points
        .into_iter()
        .map(Into::into)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_square() -> Vec<Point> {
        vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
            Point { x: 0.0, y: 10.0 },
        ]
    }

    #[test]
    fn perimeter_round_trips_through_ffi_types() {
        assert_eq!(boundary_perimeter(unit_square()), 40.0);
    }

    #[test]
    fn offset_round_trips_through_ffi_types() {
        let expanded = offset_boundary(unit_square(), 1.0);
        assert_eq!(expanded.len(), 4);
        assert!((expanded[0].x - -1.0).abs() < 1e-6);
        assert!((expanded[0].y - -1.0).abs() < 1e-6);
    }
}
