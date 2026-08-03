import Foundation

/// One cuttable piece of a garment, mirroring `patruin_pattern::PatternPiece`.
public struct PatternPiece: Identifiable, Equatable, Codable, Sendable {
    public var id: UUID
    public var name: String
    public var boundary: PatternBoundary
    public var seamAllowanceMM: Double
    public var material: Material?

    public init(
        id: UUID = UUID(),
        name: String,
        boundary: PatternBoundary,
        seamAllowanceMM: Double = 10.0,
        material: Material? = nil
    ) {
        self.id = id
        self.name = name
        self.boundary = boundary
        self.seamAllowanceMM = seamAllowanceMM
        self.material = material
    }
}

/// A garment project: its pieces, mirroring `patruin_pattern::Project`.
public struct Project: Identifiable, Equatable, Codable, Sendable {
    public var id: UUID
    public var name: String
    public var pieces: [PatternPiece]

    public init(id: UUID = UUID(), name: String, pieces: [PatternPiece] = []) {
        self.id = id
        self.name = name
        self.pieces = pieces
    }

    public var totalPerimeterMM: Double {
        pieces.reduce(0) { $0 + $1.boundary.perimeter }
    }
}
