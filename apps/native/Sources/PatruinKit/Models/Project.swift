import Foundation

/// Everything that can go wrong assembling a pattern piece.
/// Mirrors `patruin_pattern::PatternError`.
public enum PatternError: Error, Equatable, Sendable {
    /// A seam allowance must be a finite, non-negative number of
    /// millimeters. A negative one doesn't trim the piece — it drives the
    /// offset inward past its own edges and yields a larger,
    /// winding-inverted outline.
    case invalidSeamAllowance(valueMM: Double)
    /// The underlying geometry couldn't produce a cut line.
    case geometry(GeometryError)
}

extension PatternError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .invalidSeamAllowance(let valueMM):
            "seam allowance \(valueMM)mm must be finite and non-negative"
        case .geometry(let err):
            err.localizedDescription
        }
    }
}

/// One cuttable piece of a garment, mirroring `patruin_pattern::PatternPiece`.
///
/// `seamAllowanceMM` is a private store behind a validating setter, not a
/// bare `var`, for the same reason the Rust engine's field is private: an
/// unvalidated allowance is the difference between a garment that fits and
/// one cut nine times too large.
public struct PatternPiece: Identifiable, Equatable, Sendable {
    public var id: UUID
    public var name: String
    public var boundary: PatternBoundary
    private var storedSeamAllowanceMM: Double
    public var material: Material?

    /// A 10mm (1cm) seam allowance is the default starting point — a
    /// common industry convention, freely overridable per piece.
    public static let defaultSeamAllowanceMM = 10.0

    public var seamAllowanceMM: Double { storedSeamAllowanceMM }

    public init(
        id: UUID = UUID(),
        name: String,
        boundary: PatternBoundary,
        seamAllowanceMM: Double = PatternPiece.defaultSeamAllowanceMM,
        material: Material? = nil
    ) throws {
        self.id = id
        self.name = name
        self.boundary = boundary
        self.storedSeamAllowanceMM = try Self.validate(seamAllowanceMM)
        self.material = material
    }

    private static func validate(_ valueMM: Double) throws -> Double {
        guard valueMM.isFinite, valueMM >= 0 else {
            throw PatternError.invalidSeamAllowance(valueMM: valueMM)
        }
        return valueMM
    }

    /// Sets the seam allowance, rejecting values that cannot describe cloth.
    public mutating func setSeamAllowanceMM(_ valueMM: Double) throws {
        storedSeamAllowanceMM = try Self.validate(valueMM)
    }

    /// The outline including seam allowance — what actually gets cut.
    public func cutBoundary() throws -> PatternBoundary {
        do {
            return try boundary.offset(storedSeamAllowanceMM)
        } catch let error as GeometryError {
            throw PatternError.geometry(error)
        }
    }
}

extension PatternPiece: Codable {
    // This document shape is for Swift-to-Swift round-tripping only (local
    // persistence, in-memory undo snapshots) — it does not yet match the
    // Rust engine's wire format, because `id` has no counterpart on the
    // Rust side. Reconciling that is the identity-model gap the README
    // calls out; not attempted here.
    private enum CodingKeys: String, CodingKey {
        case id, name, boundary, seamAllowanceMM, material
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        try self.init(
            id: container.decode(UUID.self, forKey: .id),
            name: container.decode(String.self, forKey: .name),
            boundary: container.decode(PatternBoundary.self, forKey: .boundary),
            seamAllowanceMM: container.decode(Double.self, forKey: .seamAllowanceMM),
            material: container.decodeIfPresent(Material.self, forKey: .material)
        )
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(name, forKey: .name)
        try container.encode(boundary, forKey: .boundary)
        try container.encode(storedSeamAllowanceMM, forKey: .seamAllowanceMM)
        try container.encodeIfPresent(material, forKey: .material)
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
