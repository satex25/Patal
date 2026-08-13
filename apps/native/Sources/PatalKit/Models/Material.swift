import Foundation

/// How a material falls and moves when worn.
public enum Drape: String, Codable, CaseIterable, Sendable {
    case stiff, structured, fluid, liquid
}

/// How much a material resists bending or crushing.
public enum Rigidity: String, Codable, CaseIterable, Sendable {
    case soft, medium, firm, rigid
}

/// A single material definition, mirroring `patal_materials::Material`.
///
/// `id` is the same identity the Rust engine mints as `MaterialId` — a UUID,
/// a plain string on the wire on both sides. It used to be a Swift-only
/// invention with no Rust counterpart, which is what made the document shape
/// Swift-to-Swift only. That half of the identity gap is now closed.
///
/// A piece references a material by this id rather than embedding a copy, so
/// editing a material in a library is immediately true for every piece cut
/// from it instead of leaving stale duplicates behind.
public struct Material: Identifiable, Equatable, Codable, Sendable {
    public var id: UUID
    public var name: String
    public var weightGSM: Double?
    public var thicknessMM: Double?
    public var stretchPercent: Double?
    public var drape: Drape
    public var rigidity: Rigidity
    public var surfaceTexture: String
    public var durabilityNotes: String
    public var layerCompatibility: [String]
    public var stitchRecommendations: [String]
    public var reinforcementRequirements: [String]
    public var manufacturingConsiderations: [String]

    /// Swift spells these camelCase and Rust snake_case, and the wire format
    /// has to pick one. It picks snake_case, matching what `PatternPiece`
    /// already emits (`seam_allowance_mm`) and what every other engine type
    /// produces — so Swift maps and Rust does not rename. Without this the
    /// two sides encode the same material into documents neither can read.
    private enum CodingKeys: String, CodingKey {
        case id
        case name
        case weightGSM = "weight_gsm"
        case thicknessMM = "thickness_mm"
        case stretchPercent = "stretch_percent"
        case drape
        case rigidity
        case surfaceTexture = "surface_texture"
        case durabilityNotes = "durability_notes"
        case layerCompatibility = "layer_compatibility"
        case stitchRecommendations = "stitch_recommendations"
        case reinforcementRequirements = "reinforcement_requirements"
        case manufacturingConsiderations = "manufacturing_considerations"
    }

    public init(
        id: UUID = UUID(),
        name: String,
        weightGSM: Double? = nil,
        thicknessMM: Double? = nil,
        stretchPercent: Double? = nil,
        drape: Drape = .structured,
        rigidity: Rigidity = .medium,
        surfaceTexture: String = "",
        durabilityNotes: String = "",
        layerCompatibility: [String] = [],
        stitchRecommendations: [String] = [],
        reinforcementRequirements: [String] = [],
        manufacturingConsiderations: [String] = []
    ) {
        self.id = id
        self.name = name
        self.weightGSM = weightGSM
        self.thicknessMM = thicknessMM
        self.stretchPercent = stretchPercent
        self.drape = drape
        self.rigidity = rigidity
        self.surfaceTexture = surfaceTexture
        self.durabilityNotes = durabilityNotes
        self.layerCompatibility = layerCompatibility
        self.stitchRecommendations = stitchRecommendations
        self.reinforcementRequirements = reinforcementRequirements
        self.manufacturingConsiderations = manufacturingConsiderations
    }
}
