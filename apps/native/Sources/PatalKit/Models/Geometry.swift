import Foundation

/// A point in pattern space, in millimeters.
///
/// This mirrors `patal_geometry::Point2` in the Rust engine. It is a
/// hand-written interim model — once the engine is linked in as an
/// XCFramework, the UI layer will talk to the generated uniffi types
/// directly and this file's role narrows to view-model conveniences.
public struct Point2: Equatable, Codable, Hashable, Sendable {
    public var x: Double
    public var y: Double

    public init(x: Double, y: Double) {
        self.x = x
        self.y = y
    }

    /// Euclidean distance. Uses `hypot` rather than `(dx*dx + dy*dy).squareRoot()`
    /// so large coordinates don't overflow to infinity and small ones don't
    /// underflow to zero — the same fix the Rust engine's `Point2::distance`
    /// carries.
    public func distance(to other: Point2) -> Double {
        Foundation.hypot(x - other.x, y - other.y)
    }

    var isFinite: Bool {
        x.isFinite && y.isFinite
    }
}

/// What can go wrong *constructing* a `PatternBoundary` on this side.
///
/// Deliberately narrower than `patal_geometry::GeometryError`. The offset
/// failures (collapse, self-intersection, indeterminate winding, degenerate
/// edge, non-finite distance) are not represented here because this package
/// no longer computes an offset — see the note on `PatternBoundary`. When the
/// engine is linked in, its errors arrive as generated uniffi types and
/// nothing here has to be kept in step by hand.
public enum GeometryError: Error, Equatable, Sendable {
    case nonFiniteCoordinate(index: Int)
    case tooFewPoints(count: Int)
}

extension GeometryError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .nonFiniteCoordinate(let index):
            "point \(index) has a non-finite coordinate"
        case .tooFewPoints(let count):
            "a closed boundary needs at least 3 distinct points, got \(count)"
        }
    }
}

/// The closed outline of a pattern piece, as a straight-edged polygon.
///
/// Mirrors `patal_geometry::PatternBoundary`'s *shape and construction
/// contract*, not its mathematics: at least 3 points, all finite, no two
/// consecutive points equal, and a private store behind a read-only accessor
/// so the invariant holds for the value's whole life rather than only at its
/// birth. Decoding routes through `init(points:)` for the same reason Rust
/// routes it through `try_from`.
///
/// # What this type deliberately does not do
///
/// It does not offset, and it does not compute winding, signed area, or
/// self-intersection. Those were a 368-line hand-maintained port of the Rust
/// kernel, and a second implementation of a cutting line is a liability: if
/// the two ever disagree, one of them quietly cuts cloth wrong. Nothing
/// shipped depended on it — there is no Xcode project in this repo, and the
/// port's only caller was its own test suite — so it was deleted rather than
/// pinned in place with a conformance corpus that would have taxed every
/// future change to the engine's error surface.
///
/// Seam-allowance geometry belongs to `patal-geometry` and reaches this
/// package through uniffi bindings on Mac day. Until then, this is a model,
/// not an engine. See `docs/adr/ADR-003-curve-representation.md`.
public struct PatternBoundary: Equatable, Sendable {
    private var storedPoints: [Point2]

    public var points: [Point2] { storedPoints }

    /// Validates and normalizes a point list into a closed boundary.
    /// Consecutive duplicate points are dropped, as is a closing point that
    /// repeats the first — the same normalization `PatternBoundary::new`
    /// does in Rust, for the same reason: a duplicate point used to divide
    /// by a zero-length edge and poison the whole boundary with NaN.
    public init(points: [Point2]) throws {
        for (index, point) in points.enumerated() {
            guard point.isFinite else {
                throw GeometryError.nonFiniteCoordinate(index: index)
            }
        }

        let deduped = Self.dedupClosed(points)
        guard deduped.count >= 3 else {
            throw GeometryError.tooFewPoints(count: deduped.count)
        }

        storedPoints = deduped
    }

    private static func dedupClosed(_ points: [Point2]) -> [Point2] {
        var out: [Point2] = []
        out.reserveCapacity(points.count)
        for point in points where out.last != point {
            out.append(point)
        }
        while out.count > 1, out.first == out.last {
            out.removeLast()
        }
        return out
    }

    /// Total edge length. A sum of distances with no error surface and no
    /// cutting-line consequence, so it stays here as a view-model convenience
    /// rather than routing a display string through the FFI.
    public var perimeter: Double {
        let n = storedPoints.count
        return (0..<n).reduce(0.0) { total, i in
            total + storedPoints[i].distance(to: storedPoints[(i + 1) % n])
        }
    }
}

extension PatternBoundary: Codable {
    /// Wire format is a bare `[Point2]`, matching the Rust engine's
    /// `#[serde(try_from = "Vec<Point2>", into = "Vec<Point2>")]` exactly
    /// — a document either side writes, the other can read.
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let points = try container.decode([Point2].self)
        try self.init(points: points)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(storedPoints)
    }
}
