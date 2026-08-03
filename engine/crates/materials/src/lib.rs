//! The material system: physical characteristics that inform construction,
//! not just appearance.

/// How a material falls and moves when worn — a construction concern, not
/// just a visual one (drape affects seam placement, ease, and silhouette).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drape {
    Stiff,
    Structured,
    Fluid,
    Liquid,
}

/// How much a material resists bending or crushing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rigidity {
    Soft,
    Medium,
    Firm,
    Rigid,
}

/// A single material definition. Every field beyond `name` is optional or
/// defaultable so partially-specified materials (a designer sketching with a
/// placeholder fabric) remain valid.
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Default)]
pub struct MaterialLibrary {
    materials: Vec<Material>,
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
        let found = lib.find_by_name("Silk Charmeuse").expect("material present");
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
}
