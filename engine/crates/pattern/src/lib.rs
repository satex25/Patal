//! The pattern layer: pieces, materials, and measurements combined into a
//! project.
//!
//! The memorandum describes patterns as "a living system composed of
//! interconnected relationships" where edits propagate throughout a project.
//! This crate currently models the data those relationships will run over
//! (pieces, measurements, materials); the constraint/propagation solver
//! itself is a deliberately separate, larger milestone and is not yet
//! implemented here.

use patruin_geometry::PatternBoundary;
use patruin_materials::Material;

/// A single named measurement (e.g. "bust", "waist"), stored in millimeters.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    pub name: String,
    pub value_mm: f64,
}

/// One cuttable piece of a garment: its outline, seam allowance, and the
/// material it will be cut from.
#[derive(Debug, Clone)]
pub struct PatternPiece {
    pub name: String,
    pub boundary: PatternBoundary,
    pub seam_allowance_mm: f64,
    pub material: Option<Material>,
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

    /// The outline including seam allowance — what actually gets cut.
    pub fn cut_boundary(&self) -> PatternBoundary {
        self.boundary.offset(self.seam_allowance_mm)
    }
}

/// A garment project: its pieces and the body measurements driving them.
#[derive(Debug, Default)]
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
    }

    #[test]
    fn piece_has_default_seam_allowance() {
        let piece = PatternPiece::new("Front Bodice", square_boundary(200.0));
        assert_eq!(piece.seam_allowance_mm, 10.0);
        assert!(piece.cut_boundary().perimeter() > piece.boundary.perimeter());
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
}
