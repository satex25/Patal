//! The material system: physical characteristics that inform construction,
//! not just appearance.

use serde::{Deserialize, Serialize};

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

    pub fn add(&mut self, material: Material) {
        self.materials.push(material);
    }

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
