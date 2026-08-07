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

/// Everything that can go wrong building or offsetting a `PatternBoundary`.
///
/// Mirrors `patal_geometry::GeometryError` in the Rust engine variant for
/// variant — see that type's doc comments for why each one exists. Kept in
/// sync by hand because this whole file is a hand-written mirror, not a
/// binding; see the architecture note in the top-level README.
public enum GeometryError: Error, Equatable, Sendable {
    case nonFiniteCoordinate(index: Int)
    case tooFewPoints(count: Int)
    case degenerateEdge(index: Int)
    case nonFiniteDistance(distanceMM: Double)
    case indeterminateWinding
    case offsetCollapsed(distanceMM: Double)
    case offsetSelfIntersects(distanceMM: Double)
}

extension GeometryError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .nonFiniteCoordinate(let index):
            "point \(index) has a non-finite coordinate"
        case .tooFewPoints(let count):
            "a closed boundary needs at least 3 distinct points, got \(count)"
        case .degenerateEdge(let index):
            "edge \(index) has zero length and no direction"
        case .nonFiniteDistance(let distanceMM):
            "offset distance \(distanceMM) is not finite"
        case .indeterminateWinding:
            "boundary encloses no area, so its winding is undetermined"
        case .offsetCollapsed(let distanceMM):
            "offsetting by \(distanceMM)mm collapsed the boundary through itself"
        case .offsetSelfIntersects(let distanceMM):
            "offsetting by \(distanceMM)mm produced a self-intersecting outline"
        }
    }
}

/// Which way a boundary is wound, or that the question has no answer.
/// Mirrors `patal_geometry::Winding`.
public enum Winding: Equatable, Sendable {
    case counterClockwise
    case clockwise
    /// The boundary encloses no measurable area — a fully collinear point
    /// list, or a figure-eight whose lobes cancel exactly.
    case degenerate
}

/// The closed outline of a pattern piece, as a straight-edged polygon.
///
/// Mirrors `patal_geometry::PatternBoundary`, including its invariant:
/// construction is the only way in, so every `PatternBoundary` in existence
/// has at least 3 points, all finite, with no two consecutive points equal.
/// `points` is a private store behind a read-only accessor for the same
/// reason it's private in Rust — so the invariant holds for the value's
/// whole life, not only at its birth.
///
/// Also mirrors the Rust engine's offset construction: a mitre limit so
/// sharp corners bevel instead of flying off to infinity, and validity
/// checks (winding, edge-direction reversal, self-intersection) so a
/// collapsed or crossing result is a thrown `GeometryError`, never a
/// polygon that merely looks plausible. Until this file is deleted in
/// favor of the real uniffi bindings, this is the seam-allowance math for
/// three of Pātāl's four target platforms — see the README.
public struct PatternBoundary: Equatable, Sendable {
    private var storedPoints: [Point2]

    // Public: a default-parameter-value expression on the public
    // `offset(_:miterLimit:)` must be at least as visible as the function
    // itself, since an external caller has to be able to resolve it too.
    public static let defaultMiterLimit = 4.0
    private static let joinMergeFraction = 1e-9
    private static let areaEpsilon = 1e-12
    private static let sinEpsilon = 1e-9

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

    public var perimeter: Double {
        let n = storedPoints.count
        return (0..<n).reduce(0.0) { total, i in
            total + storedPoints[i].distance(to: storedPoints[(i + 1) % n])
        }
    }

    public var signedArea: Double {
        let n = storedPoints.count
        let sum = (0..<n).reduce(0.0) { total, i in
            let a = storedPoints[i]
            let b = storedPoints[(i + 1) % n]
            return total + (a.x * b.y - b.x * a.y)
        }
        return sum / 2.0
    }

    /// Which way the boundary is wound, in standard math orientation (y-up).
    ///
    /// Returns `.degenerate` rather than guessing when the enclosed area is
    /// indistinguishable from zero — a bare `> 0` on the shoelace sum
    /// silently reports "clockwise" for those, which flips the seam
    /// allowance inward on the whole piece.
    public var winding: Winding {
        let area = signedArea
        let scale = perimeter
        let tolerance = scale * scale * Self.areaEpsilon
        if area > tolerance {
            return .counterClockwise
        } else if area < -tolerance {
            return .clockwise
        } else {
            return .degenerate
        }
    }

    /// Whether any two non-adjacent edges cross. `O(n^2)`, the right trade
    /// at pattern-piece sizes (tens to low hundreds of vertices).
    public func selfIntersects() -> Bool {
        let n = storedPoints.count
        for i in 0..<n {
            let a1 = storedPoints[i]
            let a2 = storedPoints[(i + 1) % n]
            for j in (i + 1)..<n {
                if (i + 1) % n == j || (j + 1) % n == i { continue }
                let b1 = storedPoints[j]
                let b2 = storedPoints[(j + 1) % n]
                if Self.segmentsProperlyIntersect(a1, a2, b1, b2) {
                    return true
                }
            }
        }
        return false
    }

    /// Offsets every edge along its outward normal by `distanceMM`, then
    /// re-intersects consecutive edges to find the new vertices — the
    /// seam-allowance construction. A positive distance expands the
    /// boundary regardless of winding, a negative one insets it.
    ///
    /// The result is checked before it's returned: an offset that collapses
    /// the piece through itself, or produces a crossing outline, throws
    /// rather than returning a polygon that merely looks valid.
    public func offset(
        _ distanceMM: Double,
        miterLimit: Double = PatternBoundary.defaultMiterLimit
    ) throws -> PatternBoundary {
        guard distanceMM.isFinite else {
            throw GeometryError.nonFiniteDistance(distanceMM: distanceMM)
        }
        if distanceMM == 0 {
            return self
        }

        let currentWinding = winding
        let orientation: Double
        switch currentWinding {
        case .counterClockwise: orientation = 1.0
        case .clockwise: orientation = -1.0
        case .degenerate: throw GeometryError.indeterminateWinding
        }

        let n = storedPoints.count
        let push = orientation * distanceMM
        var offsetEdges: [(Point2, Point2)] = []
        offsetEdges.reserveCapacity(n)

        for i in 0..<n {
            let a = storedPoints[i]
            let b = storedPoints[(i + 1) % n]
            let edge = Point2(x: b.x - a.x, y: b.y - a.y)
            let len = Foundation.hypot(edge.x, edge.y)
            // Rotating the edge direction by -90 degrees gives the outward
            // normal for a counter-clockwise polygon; `orientation` flips
            // it outward for clockwise ones.
            let normal = Point2(x: edge.y / len, y: -edge.x / len)
            guard normal.isFinite else {
                throw GeometryError.degenerateEdge(index: i)
            }
            offsetEdges.append((
                Point2(x: a.x + normal.x * push, y: a.y + normal.y * push),
                Point2(x: b.x + normal.x * push, y: b.y + normal.y * push)
            ))
        }

        let miterCap = abs(miterLimit) * abs(distanceMM)
        // Where each source vertex's join landed in `newPoints`. A bevel
        // emits two points, so this is a range rather than an index.
        var joins: [(Int, Int)] = []
        joins.reserveCapacity(n)
        var newPoints: [Point2] = []
        newPoints.reserveCapacity(n)

        for i in 0..<n {
            let prev = offsetEdges[(i + n - 1) % n]
            let curr = offsetEdges[i]
            let vertex = storedPoints[i]
            let first = newPoints.count

            if let joined = Self.lineIntersection(prev, curr), joined.distance(to: vertex) <= miterCap {
                // A mitre we can afford: the two offset edges meet at a
                // point close enough to the original corner to be a real
                // corner.
                newPoints.append(joined)
            } else if prev.1.distance(to: curr.0) <= Self.joinMergeFraction * abs(distanceMM) {
                newPoints.append(curr.0)
            } else {
                // Either the mitre ran away (an acute point) or the edges
                // are parallel and never meet. Both are answered by a
                // bevel: walk the end of the previous offset edge to the
                // start of this one. Never fabricate a vertex that lies on
                // neither line.
                newPoints.append(prev.1)
                newPoints.append(curr.0)
            }

            joins.append((first, newPoints.count - 1))
        }

        // The validity test that matters: an offset edge must still run the
        // same way as the edge it came from. When an inset eats more than
        // the piece has to give, opposite edges swap places and every edge
        // reverses — and because that's a 180-degree rotation, the winding
        // and the area sign both survive it, so checking those would pass a
        // 20mm square "inset" by 15mm through as a plausible 10mm square.
        for i in 0..<n {
            let a = storedPoints[i]
            let b = storedPoints[(i + 1) % n]
            let source = Point2(x: b.x - a.x, y: b.y - a.y)

            let start = newPoints[joins[i].1]
            let end = newPoints[joins[(i + 1) % n].0]
            let offsetDirection = Point2(x: end.x - start.x, y: end.y - start.y)

            if source.x * offsetDirection.x + source.y * offsetDirection.y <= 0.0 {
                throw GeometryError.offsetCollapsed(distanceMM: distanceMM)
            }
        }

        let offsetBoundary: PatternBoundary
        do {
            offsetBoundary = try PatternBoundary(points: newPoints)
        } catch {
            throw GeometryError.offsetCollapsed(distanceMM: distanceMM)
        }

        if offsetBoundary.selfIntersects() {
            throw GeometryError.offsetSelfIntersects(distanceMM: distanceMM)
        }

        return offsetBoundary
    }

    /// Intersection of two infinite lines, each defined by a pair of
    /// points. Returns `nil` when they're parallel to within `sinEpsilon`
    /// — normalized against edge length, so the parallelism decision
    /// depends on the actual angle between the edges rather than on their
    /// scale.
    private static func lineIntersection(_ l1: (Point2, Point2), _ l2: (Point2, Point2)) -> Point2? {
        let (p1, p2) = l1
        let (p3, p4) = l2
        let d1 = Point2(x: p2.x - p1.x, y: p2.y - p1.y)
        let d2 = Point2(x: p4.x - p3.x, y: p4.y - p3.y)
        let denom = d1.x * d2.y - d1.y * d2.x

        let scale = Foundation.hypot(d1.x, d1.y) * Foundation.hypot(d2.x, d2.y)
        guard scale != 0, abs(denom / scale) >= sinEpsilon else {
            return nil
        }

        let t = ((p3.x - p1.x) * d2.y - (p3.y - p1.y) * d2.x) / denom
        let intersection = Point2(x: p1.x + t * d1.x, y: p1.y + t * d1.y)
        return intersection.isFinite ? intersection : nil
    }

    private static func cross(_ o: Point2, _ a: Point2, _ b: Point2) -> Double {
        (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
    }

    /// Whether two segments cross at an interior point of both. Touching
    /// endpoints and collinear overlap don't count — adjacent polygon
    /// edges always share a vertex, and that isn't a self-intersection.
    private static func segmentsProperlyIntersect(
        _ p1: Point2, _ p2: Point2, _ p3: Point2, _ p4: Point2
    ) -> Bool {
        let d1 = cross(p3, p4, p1)
        let d2 = cross(p3, p4, p2)
        let d3 = cross(p1, p2, p3)
        let d4 = cross(p1, p2, p4)

        return ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0))
            && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0))
    }
}

extension PatternBoundary: Codable {
    /// Wire format is a bare `[Point2]`, matching the Rust engine's
    /// `#[serde(try_from = "Vec<Point2>", into = "Vec<Point2>")]` exactly
    /// — a document either mirror writes, the other can read.
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
