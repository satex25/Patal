import Foundation

/// A point in pattern space, in millimeters.
///
/// This mirrors `patruin_geometry::Point2` in the Rust engine. It is a
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

    public func distance(to other: Point2) -> Double {
        (pow(x - other.x, 2) + pow(y - other.y, 2)).squareRoot()
    }
}

/// The closed outline of a pattern piece, as a straight-edged polygon.
/// See the Rust `PatternBoundary` for the authoritative implementation
/// (including seam-allowance offsetting); this mirror currently exposes
/// only what the UI needs to display a piece.
public struct PatternBoundary: Equatable, Codable, Sendable {
    public var points: [Point2]

    public init(points: [Point2]) {
        self.points = points
    }

    public var perimeter: Double {
        guard points.count >= 2 else { return 0 }
        return (0..<points.count).reduce(0.0) { total, i in
            total + points[i].distance(to: points[(i + 1) % points.count])
        }
    }
}
