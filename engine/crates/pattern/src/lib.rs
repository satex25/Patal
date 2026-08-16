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

use patal_geometry::{GeometryError, PatternBoundary, Point2};
use patal_materials::{Material, MaterialId, MaterialLibrary};
use serde::{Deserialize, Serialize};

mod grain;

pub use grain::GrainLine;

/// The document schema this build writes and understands.
///
/// Bumped when the shape of a saved project changes incompatibly. It is
/// here from the start because retrofitting a version field onto files that
/// already exist means guessing what version an unversioned file was — and
/// the cost of carrying it before it is needed is one integer.
///
/// It is *not* here because the format is being settled forever. Grading,
/// darts, notches, grainlines and the constraint solver are all unbuilt, so
/// version 2 is close to certain. That is the situation this field exists
/// for, not an argument against it.
pub const SCHEMA_VERSION: u32 = 1;

/// Everything that can go wrong assembling a pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternError {
    /// A seam allowance must be a finite, non-negative number of millimeters.
    /// A negative one does not trim the piece — it drives the offset inward
    /// past its own edges and yields a larger, winding-inverted outline.
    InvalidSeamAllowance { value_mm: f64 },
    /// The underlying geometry could not produce a cut line.
    Geometry(GeometryError),
    /// A piece references a material that is not in the project's library.
    ///
    /// Loudly, rather than quietly becoming `None`: a piece that silently
    /// forgets its material is a piece that gets cut from the wrong cloth,
    /// and the person who finds out is the person holding the scissors.
    MaterialNotFound { piece: String, id: MaterialId },
    /// A document written by a newer build than this one.
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    /// A grain line whose angle or anchor is not a usable number.
    InvalidGrainLine { field: &'static str, value: f64 },
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSeamAllowance { value_mm } => write!(
                f,
                "seam allowance {value_mm}mm must be finite and non-negative"
            ),
            Self::Geometry(err) => write!(f, "{err}"),
            Self::MaterialNotFound { piece, id } => write!(
                f,
                "piece \"{piece}\" references material {id}, which is not in this project"
            ),
            Self::UnsupportedSchemaVersion { found, supported } => write!(
                f,
                "this document is schema version {found}; this build understands \
                 version {supported}. It was written by a newer version of Pātāl."
            ),
            Self::InvalidGrainLine { field, value } => write!(
                f,
                "grain line {field} is {value}, which is not a finite number"
            ),
        }
    }
}

impl std::error::Error for PatternError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Geometry(err) => Some(err),
            Self::InvalidSeamAllowance { .. }
            | Self::MaterialNotFound { .. }
            | Self::UnsupportedSchemaVersion { .. }
            | Self::InvalidGrainLine { .. } => None,
        }
    }
}

impl From<GeometryError> for PatternError {
    fn from(err: GeometryError) -> Self {
        Self::Geometry(err)
    }
}

/// The line someone cuts cloth along, and proof of where it came from.
///
/// This type exists to make one specific mistake impossible rather than
/// merely forbidden. Every consumer of a cut line so far — the harness
/// preview, and now `patal-export` — needs points to draw, and the cheapest
/// way to get points is to flatten and offset the outline yourself. Doing
/// that a second time is exactly the defect class the Swift offset kernel was
/// deleted to remove: two pieces of code deciding where cloth gets cut, and
/// no way to tell which one the scissors followed.
///
/// A `CutLine` has a private field and no public constructor, so it can only
/// be minted inside this crate, by [`PatternPiece::cut_boundary`]. A
/// downstream crate can read one and it cannot fabricate one. That turns
/// constraint C11 from a rule reviewers have to remember into a rule the type
/// system enforces at compile time.
///
/// It is deliberately not `Serialize`: a cut line is derived, never stored.
/// Persisting one would let a file assert a cut line that disagrees with the
/// outline and allowance sitting next to it in the same document.
#[derive(Debug, Clone, PartialEq)]
pub struct CutLine {
    piece: String,
    boundary: PatternBoundary,
}

impl CutLine {
    /// The piece this line was derived from. Carried so that an error or a
    /// printed page can name it — "the offset failed" is not an actionable
    /// message when a project holds five pieces.
    pub fn piece_name(&self) -> &str {
        &self.piece
    }

    /// The cut line as points, in millimetres.
    pub fn points(&self) -> &[Point2] {
        self.boundary.points()
    }

    /// Read-only access to the underlying boundary, for callers that want
    /// the geometry crate's own operations.
    ///
    /// Handing out a `&PatternBoundary` does not weaken the guarantee: the
    /// guarantee is that nobody outside this crate can *mint* a `CutLine`,
    /// not that the points inside one are secret.
    pub fn boundary(&self) -> &PatternBoundary {
        &self.boundary
    }

    pub fn perimeter(&self) -> f64 {
        self.boundary.perimeter()
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
/// Serializes and deserializes through a private `PatternPieceData`, the same way
/// `PatternBoundary` goes through a plain `Vec<Point2>`: the wire format is
/// the natural shape, but arriving values are re-validated by
/// [`PatternPiece::set_seam_allowance_mm`] rather than assigned directly —
/// a `.patal` file edited by hand, or written by a future version with a
/// looser rule, cannot load a piece with a seam allowance that would
/// silently invert the cut line the way the old bare `pub f64` did.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "PatternPieceData", into = "PatternPieceData")]
pub struct PatternPiece {
    pub name: String,
    pub boundary: PatternBoundary,
    seam_allowance_mm: f64,
    /// A *reference* to a material in the project's library, not a copy.
    ///
    /// This used to be an `Option<Material>`, which embedded a snapshot: an
    /// edit to a library material left every piece holding a stale
    /// duplicate, so the shareable studio libraries the memorandum
    /// describes would have silently diverged from the pieces cut with
    /// them. Resolve it through [`Project::material_for`].
    pub material: Option<MaterialId>,
}

/// The wire shape of a [`PatternPiece`] — everything the constructor plus
/// the seam-allowance setter need, and nothing that isn't also public API.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PatternPieceData {
    name: String,
    boundary: PatternBoundary,
    seam_allowance_mm: f64,
    material: Option<MaterialId>,
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
    ///
    /// The one place in the codebase a [`CutLine`] comes into existence. See
    /// that type for why the return is a newtype rather than a bare
    /// `PatternBoundary`.
    pub fn cut_boundary(&self) -> Result<CutLine, PatternError> {
        Ok(CutLine {
            piece: self.name.clone(),
            boundary: self.boundary.offset(self.seam_allowance_mm)?,
        })
    }
}

/// A garment project: its pieces and the body measurements driving them.
///
/// No invariant of its own to protect on the wire — each `PatternPiece`
/// already validates itself on deserialization, so a plain derive here is
/// enough; `Project` doesn't need to re-check what its elements guarantee.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(try_from = "ProjectData", into = "ProjectData")]
pub struct Project {
    pub name: String,
    pub pieces: Vec<PatternPiece>,
    pub measurements: Vec<Measurement>,
    /// The project owns its materials. A piece references one by id, so
    /// editing a material here is immediately true for every piece using
    /// it, which is the whole point of the change away from embedded
    /// copies.
    pub materials: MaterialLibrary,
}

/// The wire shape of a [`Project`]. Identical to the type — the validation
/// is not about the fields, it is about whether the references between them
/// resolve.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct ProjectData {
    name: String,
    pieces: Vec<PatternPiece>,
    measurements: Vec<Measurement>,
    #[serde(default)]
    materials: MaterialLibrary,
}

impl From<Project> for ProjectData {
    fn from(project: Project) -> Self {
        Self {
            name: project.name,
            pieces: project.pieces,
            measurements: project.measurements,
            materials: project.materials,
        }
    }
}

impl TryFrom<ProjectData> for Project {
    type Error = PatternError;

    fn try_from(data: ProjectData) -> Result<Self, Self::Error> {
        let project = Project {
            name: data.name,
            pieces: data.pieces,
            measurements: data.measurements,
            materials: data.materials,
        };
        project.check_material_references()?;
        Ok(project)
    }
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pieces: Vec::new(),
            measurements: Vec::new(),
            materials: MaterialLibrary::new(),
        }
    }

    /// The material a piece will be cut from, resolved against this
    /// project's library.
    ///
    /// `Ok(None)` means the piece genuinely has no material assigned yet,
    /// which is a normal state while designing. An unresolvable reference
    /// is an error, never `None` — those two situations look identical to a
    /// caller that conflates them, and only one of them is fine.
    pub fn material_for(&self, piece: &PatternPiece) -> Result<Option<&Material>, PatternError> {
        let Some(id) = piece.material else {
            return Ok(None);
        };
        self.materials
            .find_by_id(id)
            .map(Some)
            .ok_or_else(|| PatternError::MaterialNotFound {
                piece: piece.name.clone(),
                id,
            })
    }

    /// Verifies every piece's material reference resolves. Run automatically
    /// on deserialization, and available directly for a caller that has just
    /// removed a material and wants to know what it broke.
    pub fn check_material_references(&self) -> Result<(), PatternError> {
        for piece in &self.pieces {
            self.material_for(piece)?;
        }
        Ok(())
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

/// A project plus the version of the schema it was written against — what a
/// `.patal` file contains.
///
/// The envelope is separate from [`Project`] so that the version is readable
/// before anything else is interpreted. A loader that has to parse the whole
/// project to discover it cannot read a version it does not understand well
/// enough to refuse it.
///
/// This deliberately carries no file I/O. Reading and writing bytes, atomic
/// replacement, and what happens to a half-written file are a separate
/// concern from what a document *is*, and are not built yet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "DocumentData", into = "DocumentData")]
pub struct Document {
    schema_version: u32,
    pub project: Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentData {
    schema_version: u32,
    project: Project,
}

impl From<Document> for DocumentData {
    fn from(document: Document) -> Self {
        Self {
            schema_version: document.schema_version,
            project: document.project,
        }
    }
}

impl TryFrom<DocumentData> for Document {
    type Error = PatternError;

    fn try_from(data: DocumentData) -> Result<Self, Self::Error> {
        if data.schema_version != SCHEMA_VERSION {
            return Err(PatternError::UnsupportedSchemaVersion {
                found: data.schema_version,
                supported: SCHEMA_VERSION,
            });
        }
        Ok(Self {
            schema_version: data.schema_version,
            project: data.project,
        })
    }
}

impl Document {
    /// Wraps a project at the current schema version.
    pub fn new(project: Project) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            project,
        }
    }

    /// Private field with no setter: a document's version describes the
    /// shape it was written in, so letting a caller assign it would let
    /// them claim a shape they did not produce.
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use patal_geometry::Point2;
    use patal_materials::Material;

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
    fn a_cut_line_names_the_piece_it_came_from() {
        // The provenance half of the CutLine newtype. The other half — that
        // no crate outside this one can construct a CutLine — is enforced by
        // the private field and cannot be asserted from inside the crate that
        // owns it. A `CutLine { .. }` literal in `patal-export` is a
        // compile error, which is the whole point of C11 living in the type
        // system rather than in a review checklist.
        let piece = PatternPiece::new("Front Bodice", square_boundary(200.0));
        let cut = piece.cut_boundary().expect("cuts cleanly");
        assert_eq!(cut.piece_name(), "Front Bodice");
        assert_eq!(cut.points(), cut.boundary().points());
        assert_eq!(cut.perimeter(), cut.boundary().perimeter());
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
        let silk = project.materials.add(Material::new("Silk Charmeuse"));

        let mut piece = PatternPiece::new("Skirt Panel", square_boundary(300.0));
        piece.material = Some(silk);
        project.add_piece(piece);

        let found = project.find_piece("Skirt Panel").expect("piece exists");
        let material = project
            .material_for(found)
            .expect("reference resolves")
            .expect("a material is assigned");
        assert_eq!(material.name, "Silk Charmeuse");
        assert_eq!(project.total_perimeter_mm(), 1200.0);
    }

    #[test]
    fn editing_a_library_material_reaches_every_piece_using_it() {
        // The modelling flaw this change exists to fix. PatternPiece.material
        // used to be an embedded Option<Material>, so editing the library
        // left every piece holding a stale copy — and the memorandum's
        // shareable studio libraries would have silently disagreed with the
        // pieces cut from them.
        let mut project = Project::new("Wrap Dress");
        let silk = project.materials.add(Material::new("Silk Charmeuse"));

        let mut piece = PatternPiece::new("Skirt Panel", square_boundary(300.0));
        piece.material = Some(silk);
        project.add_piece(piece);

        project
            .materials
            .find_by_id_mut(silk)
            .expect("in the library")
            .weight_gsm = Some(90.0);

        let piece = project.find_piece("Skirt Panel").unwrap();
        let seen = project.material_for(piece).unwrap().unwrap();
        assert_eq!(
            seen.weight_gsm,
            Some(90.0),
            "the piece must see the edit, not a snapshot taken when it was assigned"
        );
    }

    #[test]
    fn removing_a_material_leaves_a_reference_that_reports_itself() {
        let mut project = Project::new("Wrap Dress");
        let silk = project.materials.add(Material::new("Silk Charmeuse"));
        let mut piece = PatternPiece::new("Skirt Panel", square_boundary(300.0));
        piece.material = Some(silk);
        project.add_piece(piece);

        assert!(project.check_material_references().is_ok());
        assert_eq!(
            project.materials.remove(silk).map(|m| m.name),
            Some("Silk Charmeuse".to_string())
        );

        // Not silently dropped from the piece, and not silently None.
        assert!(matches!(
            project.check_material_references(),
            Err(PatternError::MaterialNotFound { .. })
        ));

        // Adding a material back under the same name does not re-link it.
        // Identity is what binds a piece to its cloth, not a string.
        project.materials.add(Material::new("Silk Charmeuse"));
        assert!(project.check_material_references().is_err());
    }

    #[test]
    fn an_unresolvable_material_reference_is_an_error_not_a_none() {
        let mut project = Project::new("Wrap Dress");
        let orphan = Material::new("Never Added").id();

        let mut piece = PatternPiece::new("Skirt Panel", square_boundary(300.0));
        piece.material = Some(orphan);
        project.add_piece(piece);

        match project.check_material_references() {
            Err(PatternError::MaterialNotFound { piece, id }) => {
                assert_eq!(piece, "Skirt Panel");
                assert_eq!(id, orphan);
            }
            other => panic!("expected MaterialNotFound, got {other:?}"),
        }
    }

    #[test]
    fn a_piece_with_no_material_resolves_to_none_not_an_error() {
        let mut project = Project::new("Wrap Dress");
        project.add_piece(PatternPiece::new("Skirt Panel", square_boundary(300.0)));
        let piece = project.find_piece("Skirt Panel").unwrap();
        assert!(project.material_for(piece).unwrap().is_none());
    }

    #[test]
    fn a_project_with_a_dangling_reference_cannot_be_deserialized() {
        let mut project = Project::new("Wrap Dress");
        let silk = project.materials.add(Material::new("Silk Charmeuse"));
        let mut piece = PatternPiece::new("Skirt Panel", square_boundary(300.0));
        piece.material = Some(silk);
        project.add_piece(piece);

        let json = serde_json::to_string(&project).unwrap();
        // Empty the library, leaving the piece pointing at nothing.
        let broken = json.replace(&serde_json::to_string(&project.materials).unwrap(), "[]");
        let err = serde_json::from_str::<Project>(&broken).unwrap_err();
        assert!(
            err.to_string().contains("which is not in this project"),
            "{err}"
        );
    }

    #[test]
    fn a_document_carries_its_schema_version() {
        let document = Document::new(Project::new("Wrap Dress"));
        assert_eq!(document.schema_version(), SCHEMA_VERSION);

        let json = serde_json::to_string(&document).unwrap();
        assert!(json.contains("\"schema_version\":1"), "{json}");

        let restored: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.project.name, "Wrap Dress");
    }

    #[test]
    fn a_document_from_the_future_is_refused_with_a_readable_message() {
        let json = r#"{"schema_version":99,"project":{"name":"X","pieces":[],
                       "measurements":[],"materials":[]}}"#;
        let err = serde_json::from_str::<Document>(json).unwrap_err();
        assert!(
            err.to_string().contains("written by a newer version"),
            "a version we cannot read must say so plainly: {err}"
        );
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
        let silk = Material::new("Silk Charmeuse");
        piece.material = Some(silk.id());

        let json = serde_json::to_string(&piece).unwrap();
        let restored: PatternPiece = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.name, piece.name);
        assert_eq!(restored.boundary, piece.boundary);
        assert_eq!(restored.seam_allowance_mm(), 12.5);
        assert_eq!(restored.material, Some(silk.id()));
    }

    #[test]
    fn deserializing_negative_seam_allowance_is_rejected() {
        // The bare pub f64 this used to be let a hand-edited or corrupted
        // .patal file load a piece that would cut nine times too large,
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
        let silk = project.materials.add(Material::new("Silk Charmeuse"));
        let mut piece = PatternPiece::new("Skirt Panel", square_boundary(300.0));
        piece.set_seam_allowance_mm(15.0).unwrap();
        piece.material = Some(silk);
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
