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

use patal_geometry::{GeometryError, PatternBoundary, Point2, SeamPath};
use patal_materials::{Material, MaterialId, MaterialLibrary};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// A piece's stable identity, independent of its name.
///
/// Copied deliberately from `MaterialId` rather than invented: same UUID
/// backing, same `serde(transparent)` so it is a plain string on the wire and
/// `Foundation.UUID` reads it directly, same refusal to implement `Default`.
///
/// Names are not identity. Two pieces can legitimately be called "Front", and
/// grading and export both need to say *which* piece without depending on a
/// string the designer may rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PieceId(Uuid);

impl PieceId {
    /// Mints a fresh identity. Deliberately not `Default`: an id should be
    /// created where a piece is created, never conjured to fill a gap.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for PieceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
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
    /// Private, with no setter: an id describes *which* piece this is, and a
    /// caller able to assign one is a caller able to make two pieces claim to
    /// be the same piece.
    id: PieceId,
    pub name: String,
    /// The path the designer drew, not the polygon it flattens to.
    ///
    /// Public for the same reason `boundary` was: a [`SeamPath`] cannot be
    /// constructed invalid, so assignment cannot smuggle in a bad value.
    ///
    /// The polygon is *derived*, on demand, at the document's tolerance —
    /// never stored. Storing both would let a file assert an outline that
    /// disagrees with its own curves, and there would be no way to tell which
    /// one the designer meant.
    pub outline: SeamPath,
    seam_allowance_mm: f64,
    /// A *reference* to a material in the project's library, not a copy.
    ///
    /// This used to be an `Option<Material>`, which embedded a snapshot: an
    /// edit to a library material left every piece holding a stale
    /// duplicate, so the shareable studio libraries the memorandum
    /// describes would have silently diverged from the pieces cut with
    /// them. Resolve it through [`Project::material_for`].
    pub material: Option<MaterialId>,
    /// Which way the piece sits on the cloth. `None` means unspecified, which
    /// is honest: most drafted blocks do not carry one until they are laid up.
    grain: Option<GrainLine>,
}

/// The wire shape of a [`PatternPiece`] — everything the constructor plus
/// the seam-allowance setter need, and nothing that isn't also public API.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PatternPieceData {
    id: PieceId,
    name: String,
    outline: SeamPath,
    seam_allowance_mm: f64,
    material: Option<MaterialId>,
    /// `default` rather than required: an absent grain line is a piece that
    /// has not been laid up yet, which is a real state and not a broken file.
    /// Contrast `id`, which is required — see [`PieceId`].
    #[serde(default)]
    grain: Option<GrainLine>,
}

impl TryFrom<PatternPieceData> for PatternPiece {
    type Error = PatternError;

    fn try_from(data: PatternPieceData) -> Result<Self, Self::Error> {
        let mut piece = PatternPiece::new(data.name, data.outline);
        // `new` minted a fresh id; the file's is the real one. Overwriting it
        // here rather than skipping `new` keeps every other invariant that
        // constructor establishes in one place.
        piece.id = data.id;
        piece.set_seam_allowance_mm(data.seam_allowance_mm)?;
        piece.material = data.material;
        piece.grain = data.grain;
        Ok(piece)
    }
}

impl From<PatternPiece> for PatternPieceData {
    fn from(piece: PatternPiece) -> Self {
        Self {
            id: piece.id,
            name: piece.name,
            outline: piece.outline,
            seam_allowance_mm: piece.seam_allowance_mm,
            material: piece.material,
            grain: piece.grain,
        }
    }
}

impl PatternPiece {
    /// A 10mm (1cm) seam allowance is the default starting point — a common
    /// industry convention, freely overridable per piece.
    const DEFAULT_SEAM_ALLOWANCE_MM: f64 = 10.0;

    /// Builds a piece from the path the designer drew.
    pub fn new(name: impl Into<String>, outline: SeamPath) -> Self {
        Self {
            id: PieceId::new(),
            name: name.into(),
            outline,
            seam_allowance_mm: Self::DEFAULT_SEAM_ALLOWANCE_MM,
            material: None,
            grain: None,
        }
    }

    /// Builds a piece from a polygon, lifting it into an all-corner path.
    ///
    /// Every caller that used to hand over a [`PatternBoundary`] migrates in
    /// one line, and the v1→v2 document migration uses this too. The lift is
    /// bit-exact and does no float arithmetic — see
    /// [`SeamPath::from_boundary`].
    pub fn from_boundary(name: impl Into<String>, boundary: PatternBoundary) -> Self {
        Self::new(name, SeamPath::from_boundary(&boundary))
    }

    /// This piece's identity. No setter: an id describes which piece this is,
    /// and letting a caller assign one would let two pieces claim to be the
    /// same piece.
    pub fn id(&self) -> PieceId {
        self.id
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
    ///
    /// Flattens through `flatten_for_offset`, not plain `flatten`: the
    /// discretisation has to hold *after* the offset, and a boundary
    /// flattened with no knowledge of the impending offset is precisely the
    /// error that function exists to prevent.
    ///
    /// A curve that succeeds at 0.01mm and fails at 0.001mm with
    /// `OffsetSelfIntersects` is **correct behaviour, not a regression**: a
    /// chord next to a sharp corner has become shorter than the allowance,
    /// and the loud failure is the right answer. Do not weaken the check.
    ///
    /// There is deliberately no cached boundary behind a `#[serde(skip)]`
    /// field. That is an unmeasured optimisation, and the drag-loop benchmark
    /// is what decides whether it is ever worth the second source of truth.
    pub fn cut_boundary(&self, tolerance_mm: f64) -> Result<CutLine, PatternError> {
        let flattened = self
            .outline
            .flatten_for_offset(tolerance_mm, self.seam_allowance_mm)?;
        Ok(CutLine {
            piece: self.name.clone(),
            boundary: flattened.offset(self.seam_allowance_mm)?,
        })
    }

    /// Which way this piece sits on the cloth, if it has been laid up.
    pub fn grain(&self) -> Option<GrainLine> {
        self.grain
    }

    /// Sets or clears the grain line. Unlike the seam allowance there is
    /// nothing to re-validate here: a [`GrainLine`] cannot be constructed
    /// invalid, so the only states this can reach are valid ones.
    pub fn set_grain(&mut self, grain: Option<GrainLine>) {
        self.grain = grain;
    }
}

/// The flattening tolerance a new project starts with, in millimetres.
///
/// 0.01mm against ADR-003's 0.4mm industrial-cutter figure: forty times finer
/// than any cutter can execute and far finer than cloth can hold. The last
/// wave measured this exact tolerance at roughly 1% of a 120Hz frame for one
/// piece's full drag path, so it is affordable on evidence rather than on
/// assertion.
///
/// There is deliberately **no upper bound**. A tolerance of 1e9 turns every
/// curve into a straight line, which is useless but not *wrong* in the
/// correct-or-loud sense, and inventing a ceiling means inventing a number.
/// Revisit if a real user ever sets one.
pub const DEFAULT_FLATTEN_TOLERANCE_MM: f64 = 0.01;

fn default_flatten_tolerance() -> f64 {
    DEFAULT_FLATTEN_TOLERANCE_MM
}

/// A garment project: its pieces and the body measurements driving them.
///
/// Each [`PatternPiece`] validates itself on deserialization, so the project
/// does not re-check what its elements guarantee. It does own two things they
/// cannot: whether their material references resolve, and the flattening
/// tolerance every derived cut line in the document is computed at.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// How finely authored curves are flattened before anything is cut.
    ///
    /// Private with a validated setter, and persisted: the tolerance is the
    /// entire contract between an authored curve and the polygon a cutter
    /// follows, so a document that did not carry it would produce a different
    /// cut line on reload. That is precisely the silent difference C1 exists
    /// to forbid.
    flatten_tolerance_mm: f64,
}

/// The wire shape of a [`Project`]. Identical to the type — the validation
/// is not about the fields, it is about whether the references between them
/// resolve.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectData {
    name: String,
    pieces: Vec<PatternPiece>,
    measurements: Vec<Measurement>,
    #[serde(default)]
    materials: MaterialLibrary,
    #[serde(default = "default_flatten_tolerance")]
    flatten_tolerance_mm: f64,
}

/// Hand-written, because the derived one produces `0.0` — a tolerance
/// `set_flatten_tolerance_mm` refuses. A derive here would mint an invalid
/// project through a path that never runs the validator, which is a C1
/// violation created by four characters nobody would think to look at.
impl Default for Project {
    fn default() -> Self {
        Self {
            name: String::new(),
            pieces: Vec::new(),
            measurements: Vec::new(),
            materials: MaterialLibrary::default(),
            flatten_tolerance_mm: DEFAULT_FLATTEN_TOLERANCE_MM,
        }
    }
}

/// Same trap, one level down: `ProjectData` is where `#[serde(default)]`
/// reaches for a default during deserialization, so the derive would fire
/// here without any obvious call site to notice it at.
impl Default for ProjectData {
    fn default() -> Self {
        Self {
            name: String::new(),
            pieces: Vec::new(),
            measurements: Vec::new(),
            materials: MaterialLibrary::default(),
            flatten_tolerance_mm: DEFAULT_FLATTEN_TOLERANCE_MM,
        }
    }
}

impl From<Project> for ProjectData {
    fn from(project: Project) -> Self {
        Self {
            name: project.name,
            pieces: project.pieces,
            measurements: project.measurements,
            materials: project.materials,
            flatten_tolerance_mm: project.flatten_tolerance_mm,
        }
    }
}

impl TryFrom<ProjectData> for Project {
    type Error = PatternError;

    fn try_from(data: ProjectData) -> Result<Self, Self::Error> {
        let mut project = Project {
            name: data.name,
            pieces: data.pieces,
            measurements: data.measurements,
            materials: data.materials,
            flatten_tolerance_mm: DEFAULT_FLATTEN_TOLERANCE_MM,
        };
        // Through the setter, not assigned: a file is exactly where an
        // invalid tolerance arrives from.
        project.set_flatten_tolerance_mm(data.flatten_tolerance_mm)?;
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
            flatten_tolerance_mm: DEFAULT_FLATTEN_TOLERANCE_MM,
        }
    }

    /// How finely this project's curves are flattened before anything derived
    /// from them is cut.
    pub fn flatten_tolerance_mm(&self) -> f64 {
        self.flatten_tolerance_mm
    }

    /// Sets the flattening tolerance, refusing values that cannot describe a
    /// curve. Reuses the geometry crate's own error so one condition has one
    /// name across both layers.
    pub fn set_flatten_tolerance_mm(&mut self, value_mm: f64) -> Result<(), PatternError> {
        if !value_mm.is_finite() || value_mm <= 0.0 {
            return Err(PatternError::Geometry(
                GeometryError::ToleranceNotPositive {
                    tolerance_mm: value_mm,
                },
            ));
        }
        self.flatten_tolerance_mm = value_mm;
        Ok(())
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

    /// The lookup that makes [`PieceId`] worth storing. Without it the field
    /// is bytes on disk; grading and export both index pieces by identity.
    pub fn find_piece_by_id(&self, id: PieceId) -> Option<&PatternPiece> {
        self.pieces.iter().find(|p| p.id == id)
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

    /// [`PatternPiece::cut_boundary`] at this project's tolerance.
    ///
    /// Two functions rather than a piece-to-project back-reference: the piece
    /// stays testable in isolation, and the project stays the ergonomic path.
    /// Every caller outside this crate should reach for this one, because it
    /// is the only route that cannot disagree with the document about how
    /// finely to flatten.
    pub fn cut_boundary(&self, piece: &PatternPiece) -> Result<CutLine, PatternError> {
        piece.cut_boundary(self.flatten_tolerance_mm)
    }

    /// The summed perimeter of every piece, at this project's tolerance.
    ///
    /// Fallible since §3.6: a piece stores a path, and turning a path into
    /// something with a perimeter is a flatten, which can fail. The old
    /// infallible `-> f64` had nowhere to put that and would have had to
    /// return a plausible number for a piece that has no perimeter at all.
    ///
    /// Plain `flatten`, not `flatten_for_offset`: nothing is being offset
    /// here, so tightening would be wrong.
    pub fn total_perimeter_mm(&self) -> Result<f64, PatternError> {
        let mut total = 0.0;
        for piece in &self.pieces {
            total += piece
                .outline
                .flatten(self.flatten_tolerance_mm)?
                .perimeter();
        }
        Ok(total)
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
    use patal_geometry::{EdgeSegment, Point2};
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

    fn square_path(side: f64) -> SeamPath {
        SeamPath::from_boundary(&square_boundary(side))
    }

    #[test]
    fn a_piece_keeps_the_curves_it_was_drawn_with() {
        // S1 and S2. The whole wave in one assertion: what goes in comes back
        // out as edges, not as a polygon someone has to re-guess.
        let start = Point2::new(0.0, 0.0);
        let outline = SeamPath::closed(
            start,
            vec![
                EdgeSegment::Cubic {
                    c1: Point2::new(15.0, -30.0),
                    c2: Point2::new(50.0, -22.0),
                    to: Point2::new(75.0, 10.0),
                },
                EdgeSegment::Line {
                    to: Point2::new(75.0, 100.0),
                },
                EdgeSegment::Line { to: start },
            ],
        )
        .expect("closes");

        let piece = PatternPiece::new("Bodice Front", outline.clone());
        let json = serde_json::to_string(&piece).expect("serializes");
        let restored: PatternPiece = serde_json::from_str(&json).expect("round trips");

        assert_eq!(restored.outline, outline);
        assert_eq!(restored.outline.edges().len(), 3);
        assert_eq!(
            restored
                .outline
                .edges()
                .iter()
                .filter(|e| matches!(e.geometry(), EdgeSegment::Cubic { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn no_polygon_appears_anywhere_in_a_serialized_piece() {
        // S3. The derived boundary is derived, never persisted — otherwise a
        // file can assert an outline that disagrees with its own curves.
        let piece = PatternPiece::new("Front", square_path(200.0));
        let json = serde_json::to_string(&piece).expect("serializes");
        assert!(
            !json.contains("boundary"),
            "a polygon reached the wire: {json}"
        );
        assert!(json.contains("outline"));
    }

    #[test]
    fn a_project_supplies_its_own_tolerance_to_the_cut_line() {
        // S5. Two routes to the same answer, so the ergonomic one cannot
        // drift from the testable one.
        let mut project = Project::new("Blouse");
        project.set_flatten_tolerance_mm(0.02).expect("valid");
        let piece = PatternPiece::new("Front", square_path(200.0));

        let via_project = project.cut_boundary(&piece).expect("cuts");
        let via_piece = piece.cut_boundary(0.02).expect("cuts");
        assert_eq!(via_project.points(), via_piece.points());
    }

    #[test]
    fn the_cut_line_is_flattened_against_the_offset_it_is_about_to_receive() {
        // The correctness upgrade this wave gets in passing. Plain flatten
        // discretises with no knowledge of the impending offset, which is
        // exactly the error flatten_for_offset exists to prevent. On a curved
        // piece with a large allowance the two disagree; if they ever stop
        // disagreeing, cut_boundary has quietly regressed to plain flatten.
        let start = Point2::new(0.0, 0.0);
        let outline = SeamPath::closed(
            start,
            vec![
                EdgeSegment::Cubic {
                    c1: Point2::new(10.0, 60.0),
                    c2: Point2::new(90.0, 60.0),
                    to: Point2::new(100.0, 0.0),
                },
                EdgeSegment::Line { to: start },
            ],
        )
        .expect("closes");

        let mut piece = PatternPiece::new("Curved", outline.clone());
        piece.set_seam_allowance_mm(20.0).expect("valid");

        // 0.1mm, not the 0.5mm the execution plan specified. Subdivision is
        // adaptive and recursive, so the point count moves in jumps: at 0.5mm
        // this curve's 1.41x tightening lands inside the same jump and both
        // routes return 17 points, which made the plan's assertion pass for
        // no reason and fail for the right one. Measured across allowance and
        // tolerance before picking these — at 0.1mm the two genuinely part.
        let tolerance = 0.1;
        let tight = piece.cut_boundary(tolerance).expect("cuts");
        let naive = outline
            .flatten(tolerance)
            .expect("flattens")
            .offset(20.0)
            .expect("offsets");

        assert_ne!(
            tight.points().len(),
            naive.points().len(),
            "cut_boundary must tighten for the offset, not flatten blind"
        );

        // And it must tighten by *this* rule, not merely by some rule. Without
        // this, swapping flatten_for_offset for a hand-rolled fudge factor
        // still passes the assertion above.
        let expected = outline
            .flatten_for_offset(tolerance, 20.0)
            .expect("flattens")
            .offset(20.0)
            .expect("offsets");
        assert_eq!(tight.points(), expected.points());
    }

    #[test]
    fn a_total_perimeter_reports_failure_rather_than_a_plausible_number() {
        // R2: the signature is fallible now because flattening is, and this
        // asserts both halves of that.
        //
        // The success half first — a square's perimeter is still its
        // perimeter once the piece stores a path rather than a polygon.
        let mut project = Project::new("Blouse");
        project.add_piece(PatternPiece::new("Front", square_path(100.0)));
        let total = project.total_perimeter_mm().expect("flattens");
        assert!((total - 400.0).abs() < 1e-9, "got {total}");

        // The failure half, which is what the name promises. A path that runs
        // out and straight back is closed, finite and perfectly constructible,
        // but it flattens to two distinct points — fewer than a polygon needs.
        // The old `-> f64` signature had nowhere to put that, so it would have
        // had to return a number for a piece that has no perimeter.
        let sliver = SeamPath::new(
            Point2::new(0.0, 0.0),
            vec![
                EdgeSegment::Line {
                    to: Point2::new(50.0, 0.0),
                },
                EdgeSegment::Line {
                    to: Point2::new(0.0, 0.0),
                },
            ],
        )
        .expect("closes exactly");
        project.add_piece(PatternPiece::new("Sliver", sliver));

        let err = project
            .total_perimeter_mm()
            .expect_err("a degenerate piece has no perimeter to report");
        assert!(
            matches!(
                err,
                PatternError::Geometry(GeometryError::TooFewPoints { count: 2 })
            ),
            "expected the geometry failure to surface intact, got {err:?}"
        );
    }

    #[test]
    fn a_default_project_carries_a_usable_tolerance() {
        // The trap this test exists for: a derived Default would produce 0.0,
        // which set_flatten_tolerance_mm refuses — so Project::default()
        // would mint a project the validator would have rejected, through a
        // path that never runs it.
        let project = Project::default();
        assert_eq!(project.flatten_tolerance_mm(), DEFAULT_FLATTEN_TOLERANCE_MM);
        assert!(project.flatten_tolerance_mm() > 0.0);
    }

    #[test]
    fn a_project_deserialized_without_a_tolerance_key_still_gets_a_valid_one() {
        // Fires through ProjectData's serde defaults, one level below the
        // obvious call site.
        let json = r#"{"name": "Blouse", "pieces": [], "measurements": []}"#;
        let project: Project = serde_json::from_str(json).expect("loads");
        assert_eq!(project.flatten_tolerance_mm(), DEFAULT_FLATTEN_TOLERANCE_MM);
    }

    #[test]
    fn a_tolerance_that_cannot_describe_a_curve_is_refused() {
        let mut project = Project::new("Blouse");
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(
                project.set_flatten_tolerance_mm(bad).is_err(),
                "{bad} accepted"
            );
        }
        assert_eq!(project.flatten_tolerance_mm(), DEFAULT_FLATTEN_TOLERANCE_MM);
    }

    #[test]
    fn a_tolerance_survives_the_file() {
        let mut project = Project::new("Blouse");
        project.set_flatten_tolerance_mm(0.05).expect("valid");
        let json = serde_json::to_string(&project).expect("serializes");
        let restored: Project = serde_json::from_str(&json).expect("loads");
        assert_eq!(restored.flatten_tolerance_mm(), 0.05);
    }

    #[test]
    fn two_pieces_never_share_an_id() {
        let a = PatternPiece::new("Front", square_path(100.0));
        let b = PatternPiece::new("Front", square_path(100.0));
        assert_ne!(a.id(), b.id(), "same name, different identity");
    }

    #[test]
    fn a_piece_is_findable_by_id_as_well_as_by_name() {
        let mut project = Project::new("Blouse");
        let piece = PatternPiece::new("Front", square_path(100.0));
        let id = piece.id();
        project.add_piece(piece);

        assert_eq!(
            project.find_piece_by_id(id).map(|p| p.name.as_str()),
            Some("Front")
        );
        assert!(project.find_piece_by_id(PieceId::new()).is_none());
    }

    #[test]
    fn an_id_survives_a_round_trip_and_is_a_plain_string_on_the_wire() {
        // A plain string, not a nested object, so Swift's Foundation.UUID
        // decodes it directly — the same treatment MaterialId got.
        let piece = PatternPiece::new("Front", square_path(100.0));
        let json = serde_json::to_value(&piece).expect("serializes");
        assert!(
            json["id"].is_string(),
            "id must be a bare string, got {}",
            json["id"]
        );

        let restored: PatternPiece = serde_json::from_value(json).expect("round trips");
        assert_eq!(restored.id(), piece.id());
    }

    #[test]
    fn piece_has_default_seam_allowance() {
        // Built through `from_boundary`, the one-line migration path every
        // polygon caller takes — including the v1→v2 document migration.
        let piece = PatternPiece::from_boundary("Front Bodice", square_boundary(200.0));
        assert_eq!(piece.seam_allowance_mm(), 10.0);
        let cut = piece
            .cut_boundary(DEFAULT_FLATTEN_TOLERANCE_MM)
            .expect("cuts cleanly");
        let sewing = piece
            .outline
            .flatten(DEFAULT_FLATTEN_TOLERANCE_MM)
            .expect("flattens");
        assert!(cut.perimeter() > sewing.perimeter());
    }

    #[test]
    fn a_cut_line_names_the_piece_it_came_from() {
        // The provenance half of the CutLine newtype. The other half — that
        // no crate outside this one can construct a CutLine — is enforced by
        // the private field and cannot be asserted from inside the crate that
        // owns it. A `CutLine { .. }` literal in `patal-export` is a
        // compile error, which is the whole point of C11 living in the type
        // system rather than in a review checklist.
        let piece = PatternPiece::new("Front Bodice", square_path(200.0));
        let cut = piece
            .cut_boundary(DEFAULT_FLATTEN_TOLERANCE_MM)
            .expect("cuts cleanly");
        assert_eq!(cut.piece_name(), "Front Bodice");
        assert_eq!(cut.points(), cut.boundary().points());
        assert_eq!(cut.perimeter(), cut.boundary().perimeter());
    }

    #[test]
    fn negative_seam_allowance_is_rejected() {
        let mut piece = PatternPiece::new("Front Bodice", square_path(200.0));
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
        let mut piece = PatternPiece::new("Front Bodice", square_path(200.0));
        assert!(piece.set_seam_allowance_mm(f64::NAN).is_err());
        assert!(piece.set_seam_allowance_mm(f64::INFINITY).is_err());
        assert_eq!(piece.seam_allowance_mm(), 10.0);
    }

    #[test]
    fn valid_seam_allowance_is_accepted() {
        let mut piece = PatternPiece::new("Front Bodice", square_path(200.0));
        piece.set_seam_allowance_mm(15.0).expect("15mm is fine");
        assert_eq!(piece.seam_allowance_mm(), 15.0);
        // Still an *exact* equality, deliberately. An all-corner path flattens
        // bit-identically to the polygon it was lifted from, so routing the
        // cut through a SeamPath must not move a single float. If this ever
        // needs an epsilon, the lift has stopped being bit-exact.
        assert_eq!(
            piece
                .cut_boundary(DEFAULT_FLATTEN_TOLERANCE_MM)
                .expect("cuts cleanly")
                .perimeter(),
            square_boundary(200.0).offset(15.0).unwrap().perimeter()
        );
    }

    #[test]
    fn project_tracks_pieces_and_material() {
        let mut project = Project::new("Wrap Dress");
        let silk = project.materials.add(Material::new("Silk Charmeuse"));

        let mut piece = PatternPiece::new("Skirt Panel", square_path(300.0));
        piece.material = Some(silk);
        project.add_piece(piece);

        let found = project.find_piece("Skirt Panel").expect("piece exists");
        let material = project
            .material_for(found)
            .expect("reference resolves")
            .expect("a material is assigned");
        assert_eq!(material.name, "Silk Charmeuse");
        assert_eq!(project.total_perimeter_mm().expect("flattens"), 1200.0);
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

        let mut piece = PatternPiece::new("Skirt Panel", square_path(300.0));
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
        let mut piece = PatternPiece::new("Skirt Panel", square_path(300.0));
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

        let mut piece = PatternPiece::new("Skirt Panel", square_path(300.0));
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
        project.add_piece(PatternPiece::new("Skirt Panel", square_path(300.0)));
        let piece = project.find_piece("Skirt Panel").unwrap();
        assert!(project.material_for(piece).unwrap().is_none());
    }

    #[test]
    fn a_project_with_a_dangling_reference_cannot_be_deserialized() {
        let mut project = Project::new("Wrap Dress");
        let silk = project.materials.add(Material::new("Silk Charmeuse"));
        let mut piece = PatternPiece::new("Skirt Panel", square_path(300.0));
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
        let mut piece = PatternPiece::new("Front Bodice", square_path(200.0));
        piece.set_seam_allowance_mm(12.5).unwrap();
        let silk = Material::new("Silk Charmeuse");
        piece.material = Some(silk.id());

        let json = serde_json::to_string(&piece).unwrap();
        let restored: PatternPiece = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.name, piece.name);
        assert_eq!(restored.outline, piece.outline);
        assert_eq!(restored.seam_allowance_mm(), 12.5);
        assert_eq!(restored.material, Some(silk.id()));
    }

    #[test]
    fn deserializing_negative_seam_allowance_is_rejected() {
        // The bare pub f64 this used to be let a hand-edited or corrupted
        // .patal file load a piece that would cut nine times too large,
        // silently. Loading one now fails instead.
        let json = r#"{
            "id": "5f5c1a7e-0f3b-4c9e-9a2d-6b8f4c1e2d70",
            "name": "Front Bodice",
            "outline": {
                "start": {"x":0.0,"y":0.0},
                "edges": [
                    {"geometry":{"kind":"line","to":{"x":200.0,"y":0.0}}},
                    {"geometry":{"kind":"line","to":{"x":200.0,"y":300.0}}},
                    {"geometry":{"kind":"line","to":{"x":0.0,"y":300.0}}},
                    {"geometry":{"kind":"line","to":{"x":0.0,"y":0.0}}}
                ]
            },
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
        let mut piece = PatternPiece::new("Skirt Panel", square_path(300.0));
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

    /// **Review instrument for the v2 shape freeze — not a regression.**
    ///
    /// Delete this at Task 8, once the shape is signed off. It exists so the
    /// operator signs off the *bytes* rather than a six-bullet prose summary
    /// of them, which is the objection that stopped the freeze last session.
    ///
    /// Run it with:
    ///
    /// ```sh
    /// cmd //c 'scripts\cargo.bat test --package patal-pattern -- --nocapture print_v2_shape'
    /// ```
    ///
    /// It prints the document and then **asserts** each of the six points
    /// being signed off. Printing alone would leave the reviewer diffing JSON
    /// by eye against prose, which is exactly how a wrong shape gets waved
    /// through; an assertion that fails is a claim the bytes do not support.
    #[test]
    fn print_v2_shape() {
        use patal_geometry::{Edge, Join};

        // A piece exercising every authored feature at once: a cubic, a
        // genuinely tangent `Smooth` join, a grain line and a material
        // reference. The join is load-bearing — `Join::Smooth` is validated,
        // so this construction only survives if the tangents really are
        // parallel and same-signed.
        //
        // Edge 0 leaves (0,0) along +x. Edge 1 is a cubic whose first control
        // point is also due +x of its start, so the incoming and outgoing
        // tangents are exactly collinear: sine is 0, not merely below the
        // 1e-9 threshold.
        let hem_start = Point2::new(0.0, 0.0);
        let bodice = SeamPath::with_joins(
            hem_start,
            vec![
                Edge::corner(EdgeSegment::Line {
                    to: Point2::new(100.0, 0.0),
                }),
                Edge::new(
                    EdgeSegment::Cubic {
                        c1: Point2::new(150.0, 0.0),
                        c2: Point2::new(200.0, 50.0),
                        to: Point2::new(200.0, 100.0),
                    },
                    Join::Smooth,
                ),
                Edge::corner(EdgeSegment::Line {
                    to: Point2::new(0.0, 100.0),
                }),
                Edge::corner(EdgeSegment::Line { to: hem_start }),
            ],
        )
        .expect("the smooth join is tangent by construction");

        let mut project = Project::new("Shape Freeze Review");
        let wool = project.materials.add(Material::new("Wool Suiting"));

        let mut front = PatternPiece::new("Bodice Front", bodice);
        front.material = Some(wool);
        front.set_grain(Some(
            GrainLine::new(15.0, Point2::new(50.0, 50.0)).expect("valid grain"),
        ));
        project.add_piece(front);

        // The second piece is deliberately bare: no grain, no material, all
        // corner joins. It is what an unlaid, unassigned piece looks like on
        // disk, and the null fields are as much a part of the shape being
        // frozen as the populated ones.
        project.add_piece(PatternPiece::new("Waistband", square_path(200.0)));

        // Non-default on purpose. 0.01 is the default, so a document written
        // at 0.01 cannot show whether the tolerance is persisted or merely
        // defaulted back on load — the two are indistinguishable in the bytes.
        project
            .set_flatten_tolerance_mm(0.25)
            .expect("0.25mm is a valid tolerance");

        let document = Document::new(project);
        let json = serde_json::to_string_pretty(&document).expect("document serialises");

        println!(
            "\n===== BEGIN v2 DOCUMENT SHAPE =====\n{json}\n===== END v2 DOCUMENT SHAPE =====\n"
        );

        // Re-read the printed bytes rather than the in-memory value. What is
        // being signed off is what a file round-trips as, and only a reload
        // proves the printed shape is one this build can actually read back.
        let reloaded: Document = serde_json::from_str(&json).expect("the printed shape reloads");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let piece = &value["project"]["pieces"][0];

        // 1. A piece stores `outline` (a SeamPath) and never a polygon.
        assert!(
            piece["outline"]["start"].is_object() && piece["outline"]["edges"].is_array(),
            "point 1: outline must be an authored path"
        );
        assert!(
            piece.get("boundary").is_none() && piece["outline"].get("points").is_none(),
            "point 1: no polygon may appear anywhere on a piece"
        );

        // 2. An edge is {geometry, join} — nested, not flat.
        let cubic_edge = &piece["outline"]["edges"][1];
        assert!(
            cubic_edge["geometry"]["kind"] == "cubic",
            "point 2: geometry must be nested under `geometry`, tagged by `kind`"
        );
        assert!(
            cubic_edge.get("kind").is_none(),
            "point 2: the edge must not carry the segment tag flattened onto itself"
        );
        assert_eq!(
            cubic_edge["join"], "smooth",
            "point 2: join sits beside geometry"
        );

        // 3. `join` may be omitted and means corner; `geometry` may not be.
        //
        // The claim holds on READ, and the round trip below proves it: a file
        // with no `join` key loads as a corner.
        let omitted = r#"{"start":{"x":0.0,"y":0.0},"edges":[
            {"geometry":{"kind":"line","to":{"x":10.0,"y":0.0}}},
            {"geometry":{"kind":"line","to":{"x":0.0,"y":0.0}}}
        ]}"#;
        let from_omitted: SeamPath =
            serde_json::from_str(omitted).expect("point 3: `join` must be omittable on read");
        assert_eq!(
            from_omitted.edges()[0].join(),
            Join::Corner,
            "point 3: an omitted join must mean corner"
        );

        // ...but it does NOT hold on WRITE, and that asymmetry is FINDING 1 in
        // the review dossier rather than something to quietly fix here. The
        // wire format is the thing being frozen, so whether Pātāl emits this
        // key is the operator's call, not the instrument's.
        //
        // This assertion pins the *current* behaviour so the dossier cannot
        // drift from the build it describes. If the operator decides corner
        // joins should be omitted, this is the line that flips.
        assert_eq!(
            piece["outline"]["edges"][0]["join"], "corner",
            "FINDING 1: Pātāl writes `join: corner` explicitly; it is omittable on \
             read but never omitted on write"
        );

        let no_geometry = r#"{"start":{"x":0.0,"y":0.0},"edges":[{"join":"corner"}]}"#;
        assert!(
            serde_json::from_str::<SeamPath>(no_geometry).is_err(),
            "point 3: `geometry` must be required"
        );

        // 4. A piece carries id (bare UUID string), grain (nullable),
        //    seam_allowance_mm, material.
        assert!(
            piece["id"]
                .as_str()
                .is_some_and(|s| Uuid::parse_str(s).is_ok()),
            "point 4: id must be a bare UUID string, not a wrapper object"
        );
        assert!(
            piece["grain"].is_object(),
            "point 4: grain present when laid up"
        );
        assert!(
            value["project"]["pieces"][1]["grain"].is_null(),
            "point 4: grain must be nullable, and null when unlaid"
        );
        assert!(
            piece["seam_allowance_mm"].is_number(),
            "point 4: seam allowance"
        );
        assert!(
            value["project"]["pieces"][1]["material"].is_null(),
            "point 4: material is an optional reference"
        );

        // 5. A project carries flatten_tolerance_mm, defaulting to 0.01.
        assert_eq!(
            value["project"]["flatten_tolerance_mm"], 0.25,
            "point 5: the authored tolerance must survive the file"
        );
        assert_eq!(
            reloaded.project.flatten_tolerance_mm(),
            0.25,
            "point 5: and must survive the reload, not default back"
        );
        assert_eq!(
            Project::default().flatten_tolerance_mm(),
            DEFAULT_FLATTEN_TOLERANCE_MM,
            "point 5: the default is what the freeze claims it is"
        );

        // 6. Deliberately absent: per-edge seam allowance (P-03), fold edges
        //    (P-05), notch anchors (P-13). The Edge container is what makes
        //    each of them a later field rather than a schema v3, so their
        //    absence now is the claim being signed.
        for absent in ["seam_allowance_mm", "fold", "notches"] {
            assert!(
                cubic_edge.get(absent).is_none(),
                "point 6: `{absent}` must not be on an edge yet"
            );
        }

        // The version is still 1: Tasks 1-7 changed the *shape*, and Task 8 is
        // what bumps the number and writes the migration. Signing this shape is
        // what unblocks that, so the reviewer should expect 1 here and not read
        // it as the shape being unchanged.
        assert_eq!(
            value["schema_version"], 1,
            "the bump to 2 is Task 8's job, not something Tasks 1-7 did"
        );
    }
}
