//! The FFI boundary: the seam between the platform-agnostic Rust engine and
//! each native front end (Swift on iOS/iPad/Mac, the Tauri desktop app on
//! Windows).
//!
//! Domain crates (`patal-geometry`, `patal-materials`, `patal-pattern`)
//! stay free of FFI concerns; this crate defines uniffi-friendly DTOs and
//! converts to/from the real domain types. The exported surface here is
//! intentionally small — it grows alongside the domain crates rather than
//! trying to expose everything up front.
//!
//! Fallible engine operations cross this boundary as errors, not as
//! plausible-looking numbers. A front end that receives NaN has no way to
//! tell a valid measurement from a corrupted one.

#![forbid(unsafe_code)]

use std::fmt;

use patal_geometry::GeometryError;

uniffi::setup_scaffolding!();

/// The error surface of the engine, as seen from Swift or TypeScript.
///
/// Flattened to a message on the foreign side: front ends should show the
/// text and let the user fix the outline, not branch on the variant.
#[derive(Debug, uniffi::Error)]
#[uniffi(flat_error)]
pub enum EngineError {
    Geometry(GeometryError),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Geometry(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for EngineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Geometry(err) => Some(err),
        }
    }
}

impl From<GeometryError> for EngineError {
    fn from(err: GeometryError) -> Self {
        Self::Geometry(err)
    }
}

/// An FFI-safe 2D point. Converts to/from `patal_geometry::Point2`.
#[derive(uniffi::Record)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl From<Point> for patal_geometry::Point2 {
    fn from(p: Point) -> Self {
        patal_geometry::Point2::new(p.x, p.y)
    }
}

impl From<patal_geometry::Point2> for Point {
    fn from(p: patal_geometry::Point2) -> Self {
        Point { x: p.x, y: p.y }
    }
}

fn boundary_from(points: Vec<Point>) -> Result<patal_geometry::PatternBoundary, EngineError> {
    Ok(patal_geometry::PatternBoundary::new(
        points.into_iter().map(Into::into).collect(),
    )?)
}

#[uniffi::export]
pub fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// The perimeter, in millimeters, of a closed polygon boundary.
#[uniffi::export]
pub fn boundary_perimeter(points: Vec<Point>) -> Result<f64, EngineError> {
    Ok(boundary_from(points)?.perimeter())
}

/// Offsets a closed polygon boundary outward by `distance_mm` — the seam
/// allowance construction, exposed across the FFI boundary.
#[uniffi::export]
pub fn offset_boundary(points: Vec<Point>, distance_mm: f64) -> Result<Vec<Point>, EngineError> {
    let offset = boundary_from(points)?.offset(distance_mm)?;
    Ok(offset.into_points().into_iter().map(Into::into).collect())
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
        assert_eq!(boundary_perimeter(unit_square()).unwrap(), 40.0);
    }

    #[test]
    fn offset_round_trips_through_ffi_types() {
        let expanded = offset_boundary(unit_square(), 1.0).expect("offsets cleanly");
        assert_eq!(expanded.len(), 4);
        assert!((expanded[0].x - -1.0).abs() < 1e-6);
        assert!((expanded[0].y - -1.0).abs() < 1e-6);
    }

    #[test]
    fn non_finite_input_crosses_the_boundary_as_an_error() {
        let poisoned = vec![
            Point { x: 0.0, y: 0.0 },
            Point {
                x: f64::NAN,
                y: 0.0,
            },
            Point { x: 10.0, y: 10.0 },
        ];
        assert!(boundary_perimeter(poisoned).is_err());
    }

    #[test]
    fn over_inset_crosses_the_boundary_as_an_error() {
        assert!(offset_boundary(unit_square(), -20.0).is_err());
    }
}
