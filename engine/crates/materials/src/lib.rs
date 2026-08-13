//! The material system: physical characteristics that inform construction,
//! not just appearance.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A material's stable identity, independent of its name.
///
/// Names are not identity: two studios both have a "Cotton Poplin", and
/// renaming one must not silently repoint every piece cut from it. On the
/// wire this is a plain UUID string, which is what Swift's
/// `Foundation.UUID` encodes to — so the identity model that had diverged
/// between the two sides now agrees by construction rather than by
/// coincidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MaterialId(Uuid);

impl MaterialId {
    /// Mints a fresh identity. Deliberately not `Default`: an id should be
    /// created where a material is created, never conjured to fill a gap.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for MaterialId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// How a material falls and moves when worn — a construction concern, not
/// just a visual one (drape affects seam placement, ease, and silhouette).
///
/// `rename_all = "snake_case"` gives Rust's `Stiff` the wire form `"stiff"`
/// — matching PatalKit's Swift `Drape` raw values exactly, so a JSON
/// document produced by either mirror deserializes cleanly in the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Drape {
    Stiff,
    Structured,
    Fluid,
    Liquid,
}

/// How much a material resists bending or crushing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rigidity {
    Soft,
    Medium,
    Firm,
    Rigid,
}

/// A single material definition. Every field beyond `name` is optional or
/// defaultable so partially-specified materials (a designer sketching with a
/// placeholder fabric) remain valid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    /// Private, and there is no setter. Identity is assigned once at
    /// creation and preserved verbatim through serialization; letting a
    /// caller reassign it would orphan every piece that references it.
    id: MaterialId,
    pub name: String,
    pub weight_gsm: Option<f64>,
    pub thickness_mm: Option<f64>,
    pub stretch_percent: Option<f64>,
    pub drape: Drape,
    pub rigidity: Rigidity,
    pub surface_texture: String,
    pub durability_notes: String,
    pub layer_compatibility: Vec<String>,
    pub stitch_recommendations: Vec<String>,
    pub reinforcement_requirements: Vec<String>,
    pub manufacturing_considerations: Vec<String>,
}

impl Material {
    /// A minimal material with just a name; every other attribute can be
    /// filled in incrementally as the design develops.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: MaterialId::new(),
            name: name.into(),
            weight_gsm: None,
            thickness_mm: None,
            stretch_percent: None,
            drape: Drape::Structured,
            rigidity: Rigidity::Medium,
            surface_texture: String::new(),
            durability_notes: String::new(),
            layer_compatibility: Vec::new(),
            stitch_recommendations: Vec::new(),
            reinforcement_requirements: Vec::new(),
            manufacturing_considerations: Vec::new(),
        }
    }

    pub fn id(&self) -> MaterialId {
        self.id
    }
}

/// A collection of materials — a studio's, brand's, or manufacturer's
/// proprietary library, or the built-in default set.
///
/// `materials` is private for encapsulation, not to protect an invariant —
/// there is none yet, so the wire format is simply the list, via
/// `#[serde(from, into)]` rather than the fallible `try_from` that a type
/// with a real invariant (`patal_geometry::PatternBoundary`) needs.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "Vec<Material>", into = "Vec<Material>")]
pub struct MaterialLibrary {
    materials: Vec<Material>,
}

impl From<Vec<Material>> for MaterialLibrary {
    fn from(materials: Vec<Material>) -> Self {
        Self { materials }
    }
}

impl From<MaterialLibrary> for Vec<Material> {
    fn from(library: MaterialLibrary) -> Self {
        library.materials
    }
}

impl MaterialLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a material and hands back its id, so the caller can reference it
    /// from a piece without having to go looking for it by name.
    pub fn add(&mut self, material: Material) -> MaterialId {
        let id = material.id();
        self.materials.push(material);
        id
    }

    /// The lookup that matters: identity, not name.
    pub fn find_by_id(&self, id: MaterialId) -> Option<&Material> {
        self.materials.iter().find(|m| m.id == id)
    }

    /// Editing access, by identity. Safe to hand out because `Material::id`
    /// is private with no setter — a caller can change what a material *is*
    /// but not which material it is, so no reference can be invalidated
    /// through this.
    pub fn find_by_id_mut(&mut self, id: MaterialId) -> Option<&mut Material> {
        self.materials.iter_mut().find(|m| m.id == id)
    }

    /// Removes a material, returning it.
    ///
    /// Note what this deliberately does not do: it does not go hunting for
    /// pieces that referenced the removed material. Those references become
    /// unresolvable, and `patal_pattern` reports that loudly when a project
    /// is loaded rather than silently dropping the material from the piece.
    pub fn remove(&mut self, id: MaterialId) -> Option<Material> {
        let index = self.materials.iter().position(|m| m.id == id)?;
        Some(self.materials.remove(index))
    }

    /// Convenience lookup for human-facing search. Ambiguous by nature —
    /// two materials may share a name — so it returns the first match and
    /// should not be used to establish a reference.
    pub fn find_by_name(&self, name: &str) -> Option<&Material> {
        self.materials.iter().find(|m| m.name == name)
    }

    pub fn len(&self) -> usize {
        self.materials.len()
    }

    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Material> {
        self.materials.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_material_has_sane_defaults() {
        let m = Material::new("Cotton Poplin");
        assert_eq!(m.name, "Cotton Poplin");
        assert_eq!(m.drape, Drape::Structured);
        assert_eq!(m.rigidity, Rigidity::Medium);
        assert!(m.weight_gsm.is_none());
    }

    #[test]
    fn library_add_and_find() {
        let mut lib = MaterialLibrary::new();
        assert!(lib.is_empty());

        let mut silk = Material::new("Silk Charmeuse");
        silk.drape = Drape::Liquid;
        silk.weight_gsm = Some(60.0);
        lib.add(silk);

        assert_eq!(lib.len(), 1);
        let found = lib
            .find_by_name("Silk Charmeuse")
            .expect("material present");
        assert_eq!(found.drape, Drape::Liquid);
        assert_eq!(found.weight_gsm, Some(60.0));
    }

    #[test]
    fn find_by_name_missing_returns_none() {
        let lib = MaterialLibrary::new();
        assert!(lib.find_by_name("Denim").is_none());
    }

    #[test]
    fn two_materials_with_the_same_name_are_not_the_same_material() {
        // Names are not identity. A studio can stock two cotton poplins from
        // different mills, and a piece cut from one must not follow an edit
        // to the other.
        let a = Material::new("Cotton Poplin");
        let b = Material::new("Cotton Poplin");
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn add_returns_the_id_needed_to_reference_it() {
        let mut lib = MaterialLibrary::new();
        let denim = Material::new("Denim");
        let expected = denim.id();

        let id = lib.add(denim);
        assert_eq!(id, expected);
        assert_eq!(lib.find_by_id(id).map(|m| m.name.as_str()), Some("Denim"));
    }

    #[test]
    fn remove_takes_the_material_out_by_identity() {
        let mut lib = MaterialLibrary::new();
        let denim = lib.add(Material::new("Denim"));
        let silk = lib.add(Material::new("Silk"));

        assert_eq!(lib.remove(denim).map(|m| m.name), Some("Denim".to_string()));
        assert_eq!(lib.len(), 1);
        assert!(lib.find_by_id(denim).is_none());
        assert!(lib.find_by_id(silk).is_some());
        assert!(
            lib.remove(denim).is_none(),
            "removing twice is not an error"
        );
    }

    #[test]
    fn an_id_survives_a_round_trip_as_a_plain_uuid_string() {
        // The wire form Swift's Foundation.UUID also produces, so the two
        // sides agree on identity by construction rather than convention.
        let material = Material::new("Denim");
        let json = serde_json::to_string(&material).unwrap();
        assert!(
            json.contains(&format!(r#""id":"{}""#, material.id())),
            "{json}"
        );

        let restored: Material = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id(), material.id());
    }

    #[test]
    fn iter_visits_all_materials() {
        let mut lib = MaterialLibrary::new();
        lib.add(Material::new("A"));
        lib.add(Material::new("B"));
        let names: Vec<&str> = lib.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, vec!["A", "B"]);
    }

    #[test]
    fn drape_and_rigidity_serialize_lowercase_matching_the_swift_mirror() {
        // PatalKit's Swift `Drape`/`Rigidity` are String-backed enums with
        // these exact lowercase raw values — keeping Rust's wire form in
        // sync means a document either side writes, the other can read.
        assert_eq!(serde_json::to_string(&Drape::Stiff).unwrap(), r#""stiff""#);
        assert_eq!(serde_json::to_string(&Rigidity::Firm).unwrap(), r#""firm""#);
        assert_eq!(
            serde_json::from_str::<Drape>(r#""liquid""#).unwrap(),
            Drape::Liquid
        );
    }

    #[test]
    fn material_round_trips_through_json() {
        let mut silk = Material::new("Silk Charmeuse");
        silk.drape = Drape::Liquid;
        silk.weight_gsm = Some(60.0);
        silk.stitch_recommendations = vec!["French seam".to_string()];

        let json = serde_json::to_string(&silk).unwrap();
        let restored: Material = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, silk);
    }

    #[test]
    fn material_library_wire_format_is_a_bare_material_array() {
        let mut lib = MaterialLibrary::new();
        lib.add(Material::new("Denim"));
        let json = serde_json::to_string(&lib).unwrap();
        assert!(json.starts_with('['), "expected a bare array, got {json}");
    }

    #[test]
    fn material_library_round_trips_through_json() {
        let mut lib = MaterialLibrary::new();
        lib.add(Material::new("Denim"));
        lib.add(Material::new("Silk Charmeuse"));

        let json = serde_json::to_string(&lib).unwrap();
        let restored: MaterialLibrary = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.len(), 2);
        assert!(restored.find_by_name("Denim").is_some());
        assert!(restored.find_by_name("Silk Charmeuse").is_some());
    }
}
