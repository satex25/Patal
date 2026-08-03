import Foundation

/// How a material falls and moves when worn.
public enum Drape: String, Codable, CaseIterable, Sendable {
    case stiff, structured, fluid, liquid
}

/// How much a material resists bending or crushing.
public enum Rigidity: String, Codable, CaseIterable, Sendable {
    case soft, medium, firm, rigid
}

/// A single material definition, mirroring `patruin_materials::Material`.
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

    public init(
        id: UUID = UUID(),
        name: String,
        weightGSM: Double? = nil,
        thicknessMM: Double? = nil,
        stretchPercent: Double? = nil,
        drape: Drape = .structured,
        rigidity: Rigidity = .medium,
        surfaceTexture: String = "",
        durabilityNotes: String = ""
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
    }
}
