//! The pattern layer: pieces, materials, and measurements combined into a
//! project.
//!
//! The memorandum describes patterns as "a living system composed of
//! interconnected relationships" where edits propagate throughout a project.
//! This crate currently models the data those relationships will run over
//! (pieces, measurements, materials); the constraint/propagation solver
//! itself is a deliberately separate, larger milestone and is not yet
//! implemented here.

#![forbid(unsafe_code)]

use std::fmt;

use patruin_geometry::{GeometryError, PatternBoundary};
use patruin_materials::Material;
use serde::{Deserialize, Serialize};

/// Everything that can go wrong assembling a pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternError {
    /// A seam allowance must be a finite, non-negative number of millimeters.
    /// A negative one does not trim the piece — it drives the offset inward
    /// past its own edges and yields a larger, winding-inverted outline.
    InvalidSeamAllowance { value_mm: f64 },
    /// The underlying geometry could not produce a cut line.
    Geometry(GeometryError),
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSeamAllowance { value_mm } => write!(
                f,
                "seam allowance {value_mm}mm must be finite and non-negative"
            ),
            Self::Geometry(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for PatternError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Geometry(err) => Some(err),
            Self::InvalidSeamAllowance { .. } => None,
        }
    }
}

impl From<GeometryError> for PatternError {
    fn from(err: GeometryError) -> Self {
        Self::Geometry(err)
    }
}

/// A single named measurement (e.g. "bust", "waist"), stored in millimeters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    pub name: String,
    pub value_mm: f64,
}

/// One cuttable piece of a garment: its outline, seam allowance, and the
/// material it will be cut from.
///
/// Serializes and deserializes through [`PatternPieceData`], the same way
/// `PatternBoundary` goes through a plain `Vec<Point2>`: the wire format is
/// the natural shape, but arriving values are re-validated by
/// [`PatternPiece::set_seam_allowance_mm`] rather than assigned directly —
/// a `.patruin` file edited by hand, or written by a future version with a
/// looser rule, cannot load a piece with a seam allowance that would
/// silently invert the cut line the way the old bare `pub f64` did.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "PatternPieceData", into = "PatternPieceData")]
pub struct PatternPiece {
    pub name: String,
    pub boundary: PatternBoundary,
    seam_allowance_mm: f64,
    pub material: Option<Material>,
}

/// The wire shape of a [`PatternPiece`] — everything the constructor plus
/// the seam-allowance setter need, and nothing that isn't also public API.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PatternPieceData {
    name: String,
    boundary: PatternBoundary,
    seam_allowance_mm: f64,
    material: Option<Material>,
}

impl TryFrom<PatternPieceData> for PatternPiece {
    type Error = PatternError;

    fn try_from(data: PatternPieceData) -> Result<Self, Self::Error> {
        let mut piece = PatternPiece::new(data.name, data.boundary);
        piece.set_seam_allowance_mm(data.seam_allowance_mm)?;
        piece.material = data.material;
        Ok(piece)
    }
}

impl From<PatternPiece> for PatternPieceData {
    fn from(piece: PatternPiece) -> Self {
        Self {
            name: piece.name,
            boundary: piece.boundary,
            seam_allowance_mm: piece.seam_allowance_mm,
            material: piece.material,
        }
    }
}

impl PatternPiece {
    /// A 10mm (1cm) seam allowance is the default starting point — a common
    /// industry convention, freely overridable per piece.
    const DEFAULT_SEAM_ALLOWANCE_MM: f64 = 10.0;

    pub fn new(name: impl Into<String>, boundary: PatternBoundary) -> Self {
        Self {
            name: name.into(),
            boundary,
            seam_allowance_mm: Self::DEFAULT_SEAM_ALLOWANCE_MM,
            material: None,
        }
    }

    pub fn seam_allowance_mm(&self) -> f64 {
        self.seam_allowance_mm
    }

    /// Sets the seam allowance, rejecting values that cannot describe cloth.
    ///
    /// The field is private precisely so this check cannot be bypassed: an
    /// unvalidated allowance is the difference between a garment that fits
    /// and one cut nine times too large.
    pub fn set_seam_allowance_mm(&mut self, value_mm: f64) -> Result<(), PatternError> {
        if !value_mm.is_finite() || value_mm < 0.0 {
            return Err(PatternError::InvalidSeamAllowance { value_mm });
        }
        self.seam_allowance_mm = value_mm;
        Ok(())
    }

    /// The outline including seam allowance — what actually gets cut.
    pub fn cut_boundary(&self) -> Result<PatternBoundary, PatternError> {
        Ok(self.boundary.offset(self.seam_allowance_mm)?)
    }
}

/// A garment project: its pieces and the body measurements driving them.
///
/// No invariant of its own to protect on the wire — each `PatternPiece`
/// already validates itself on deserialization, so a plain derive here is
/// enough; `Project` doesn't need to re-check what its elements guarantee.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub pieces: Vec<PatternPiece>,
    pub measurements: Vec<Measurement>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pieces: Vec::new(),
            measurements: Vec::new(),
        }
    }

    pub fn add_piece(&mut self, piece: PatternPiece) {
        self.pieces.push(piece);
    }

    pub fn find_piece(&self, name: &str) -> Option<&PatternPiece> {
        self.pieces.iter().find(|p| p.name == name)
    }

    /// Sets a named measurement, overwriting any existing value of the same
    /// name.
    pub fn set_measurement(&mut self, name: impl Into<String>, value_mm: f64) {
        let name = name.into();
        match self.measurements.iter_mut().find(|m| m.name == name) {
            Some(existing) => existing.value_mm = value_mm,
            None => self.measurements.push(Measurement { name, value_mm }),
        }
    }

    pub fn measurement(&self, name: &str) -> Option<f64> {
        self.measurements
            .iter()
            .find(|m| m.name == name)
            .map(|m| m.value_mm)
    }

    pub fn total_perimeter_mm(&self) -> f64 {
        self.pieces.iter().map(|p| p.boundary.perimeter()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use patruin_geometry::Point2;
    use patruin_materials::Material;

    fn square_boundary(side: f64) -> PatternBoundary {
        PatternBoundary::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(side, 0.0),
            Point2::new(side, side),
            Point2::new(0.0, side),
        ])
        .expect("square is a valid boundary")
    }

    #[test]
    fn piece_has_default_seam_allowance() {
        let piece = PatternPiece::new("Front Bodice", square_boundary(200.0));
        assert_eq!(piece.seam_allowance_mm(), 10.0);
        let cut = piece.cut_boundary().expect("cuts cleanly");
        assert!(cut.perimeter() > piece.boundary.perimeter());
    }

    #[test]
    fn negative_seam_allowance_is_rejected() {
        let mut piece = PatternPiece::new("Front Bodice", square_boundary(200.0));
        let err = piece.set_seam_allowance_mm(-1000.0).unwrap_err();
        assert_eq!(
            err,
            PatternError::InvalidSeamAllowance { value_mm: -1000.0 }
        );
        // The rejected value must not have landed.
        assert_eq!(piece.seam_allowance_mm(), 10.0);
    }

    #[test]
    fn non_finite_seam_allowance_is_rejected() {
        let mut piece = PatternPiece::new("Front Bodice", square_boundary(200.0));
        assert!(piece.set_seam_allowance_mm(f64::NAN).is_err());
        assert!(piece.set_seam_allowance_mm(f64::INFINITY).is_err());
        assert_eq!(piece.seam_allowance_mm(), 10.0);
    }

    #[test]
    fn valid_seam_allowance_is_accepted() {
        let mut piece = PatternPiece::new("Front Bodice", square_boundary(200.0));
        piece.set_seam_allowance_mm(15.0).expect("15mm is fine");
        assert_eq!(piece.seam_allowance_mm(), 15.0);
        assert_eq!(
            piece.cut_boundary().expect("cuts cleanly").perimeter(),
            square_boundary(200.0).offset(15.0).unwrap().perimeter()
        );
    }

    #[test]
    fn project_tracks_pieces_and_material() {
        let mut project = Project::new("Wrap Dress");
        let mut piece = PatternPiece::new("Skirt Panel", square_boundary(300.0));
        piece.material = Some(Material::new("Silk Charmeuse"));
        project.add_piece(piece);

        let found = project.find_piece("Skirt Panel").expect("piece exists");
        assert_eq!(found.material.as_ref().unwrap().name, "Silk Charmeuse");
        assert_eq!(project.total_perimeter_mm(), 1200.0);
    }

    #[test]
    fn measurements_can_be_set_and_overwritten() {
        let mut project = Project::new("Wrap Dress");
        project.set_measurement("bust", 900.0);
        project.set_measurement("waist", 700.0);
        project.set_measurement("bust", 910.0);

        assert_eq!(project.measurement("bust"), Some(910.0));
        assert_eq!(project.measurement("waist"), Some(700.0));
        assert_eq!(project.measurement("hip"), None);
        assert_eq!(project.measurements.len(), 2);
    }

    #[test]
    fn measurement_round_trips_through_json() {
        let m = Measurement {
            name: "bust".to_string(),
            value_mm: 900.0,
        };
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(serde_json::from_str::<Measurement>(&json).unwrap(), m);
    }

    #[test]
    fn pattern_piece_round_trips_through_json() {
        let mut piece = PatternPiece::new("Front Bodice", square_boundary(200.0));
        piece.set_seam_allowance_mm(12.5).unwrap();
        piece.material = Some(Material::new("Silk Charmeuse"));

        let json = serde_json::to_string(&piece).unwrap();
        let restored: PatternPiece = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.name, piece.name);
        assert_eq!(restored.boundary, piece.boundary);
        assert_eq!(restored.seam_allowance_mm(), 12.5);
        assert_eq!(
            restored.material.as_ref().unwrap().name,
            "Silk Charmeuse"
        );
    }

    #[test]
    fn deserializing_negative_seam_allowance_is_rejected() {
        // The bare pub f64 this used to be let a hand-edited or corrupted
        // .patruin file load a piece that would cut nine times too large,
        // silently. Loading one now fails instead.
        let json = r#"{
            "name": "Front Bodice",
            "boundary": [
                {"x":0.0,"y":0.0},{"x":200.0,"y":0.0},
                {"x":200.0,"y":300.0},{"x":0.0,"y":300.0}
            ],
            "seam_allowance_mm": -1000.0,
            "material": null
        }"#;
        let err = serde_json::from_str::<PatternPiece>(json).unwrap_err();
        assert!(err.to_string().contains("finite and non-negative"));
    }

    #[test]
    fn project_round_trips_through_json_including_nested_validation() {
        let mut project = Project::new("Wrap Dress");
        let mut piece = PatternPiece::new("Skirt Panel", square_boundary(300.0));
        piece.set_seam_allowance_mm(15.0).unwrap();
        piece.material = Some(Material::new("Silk Charmeuse"));
        project.add_piece(piece);
        project.set_measurement("waist", 700.0);

        let json = serde_json::to_string(&project).unwrap();
        let restored: Project = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.name, "Wrap Dress");
        assert_eq!(restored.measurement("waist"), Some(700.0));
        let piece = restored.find_piece("Skirt Panel").expect("piece present");
        assert_eq!(piece.seam_allowance_mm(), 15.0);
    }
}
