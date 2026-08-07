import XCTest
@testable import PatalKit

final class PatalKitTests: XCTestCase {
    private func squareBoundary(side: Double) throws -> PatternBoundary {
        try PatternBoundary(points: [
            Point2(x: 0, y: 0),
            Point2(x: side, y: 0),
            Point2(x: side, y: side),
            Point2(x: 0, y: side),
        ])
    }

    func testPerimeterOfSquare() throws {
        XCTAssertEqual(try squareBoundary(side: 10).perimeter, 40.0, accuracy: 1e-9)
    }

    func testProjectTotalPerimeterSumsPieces() throws {
        let piece = try PatternPiece(name: "Panel", boundary: squareBoundary(side: 10))
        var project = Project(name: "Test Dress")
        project.pieces.append(piece)
        XCTAssertEqual(project.totalPerimeterMM, 40.0, accuracy: 1e-9)
    }

    func testMaterialDefaults() {
        let material = Material(name: "Cotton")
        XCTAssertEqual(material.drape, .structured)
        XCTAssertEqual(material.rigidity, .medium)
        XCTAssertNil(material.weightGSM)
    }

    // MARK: - PatternBoundary construction

    func testDuplicateConsecutivePointsAreDroppedNotDividedBy() throws {
        // Mirrors the Rust engine's regression test for the bug this whole
        // port exists to not repeat: a duplicate point used to divide by a
        // zero-length edge and poison the boundary with NaN.
        let boundary = try PatternBoundary(points: [
            Point2(x: 0, y: 0),
            Point2(x: 100, y: 0),
            Point2(x: 100, y: 0),
            Point2(x: 100, y: 200),
            Point2(x: 0, y: 200),
        ])
        XCTAssertEqual(boundary.points.count, 4)
        let cut = try boundary.offset(10.0)
        XCTAssertTrue(cut.points.allSatisfy { $0.x.isFinite && $0.y.isFinite })
    }

    func testFewerThanThreePointsIsRejected() {
        XCTAssertThrowsError(
            try PatternBoundary(points: [Point2(x: 0, y: 0), Point2(x: 50, y: 0)])
        ) { error in
            XCTAssertEqual(error as? GeometryError, .tooFewPoints(count: 2))
        }
    }

    func testNonFiniteCoordinateIsRejected() {
        XCTAssertThrowsError(
            try PatternBoundary(points: [
                Point2(x: 0, y: 0),
                Point2(x: .nan, y: 0),
                Point2(x: 10, y: 10),
            ])
        ) { error in
            XCTAssertEqual(error as? GeometryError, .nonFiniteCoordinate(index: 1))
        }
    }

    // MARK: - offset

    func testOffsetExpandsSquareSymmetrically() throws {
        let expanded = try squareBoundary(side: 10).offset(1.0)
        let expected = [
            Point2(x: -1, y: -1),
            Point2(x: 11, y: -1),
            Point2(x: 11, y: 11),
            Point2(x: -1, y: 11),
        ]
        XCTAssertEqual(expanded.points.count, expected.count)
        for (actual, expected) in zip(expanded.points, expected) {
            XCTAssertEqual(actual.x, expected.x, accuracy: 1e-6)
            XCTAssertEqual(actual.y, expected.y, accuracy: 1e-6)
        }
    }

    func testAcutePointIsBevelledNotFlungAway() throws {
        // A 6-degree dart apex. Un-capped, the mitre puts the offset vertex
        // roughly 190mm from the original corner on a 200mm-tall piece.
        let spike = try PatternBoundary(points: [
            Point2(x: 0, y: 0),
            Point2(x: 100, y: 0),
            Point2(x: 100, y: 200),
            Point2(x: 94.75, y: 100),
            Point2(x: 0, y: 200),
        ])
        let distance = 10.0
        let offset = try spike.offset(distance)
        let cap = PatternBoundary.defaultMiterLimit * distance
        for point in offset.points {
            let nearest = spike.points.map { $0.distance(to: point) }.min() ?? .infinity
            XCTAssertLessThanOrEqual(nearest, cap + 1e-9)
        }
    }

    func testOverInsetReportsCollapseInsteadOfInverting() throws {
        // A 20mm square inset by 15mm used to return a valid-looking 10mm
        // square with inverted winding and a positive area, in the Rust
        // engine before it was fixed; this port carries the same guard.
        let square = try PatternBoundary(points: [
            Point2(x: 0, y: 0), Point2(x: 20, y: 0),
            Point2(x: 20, y: 20), Point2(x: 0, y: 20),
        ])
        XCTAssertThrowsError(try square.offset(-15.0)) { error in
            XCTAssertEqual(error as? GeometryError, .offsetCollapsed(distanceMM: -15.0))
        }
    }

    func testConcaveSlotNarrowerThanTheOffsetIsReported() throws {
        let slotted = try PatternBoundary(points: [
            Point2(x: 0, y: 0), Point2(x: 100, y: 0),
            Point2(x: 100, y: 60), Point2(x: 57, y: 60),
            Point2(x: 57, y: 30), Point2(x: 43, y: 30),
            Point2(x: 43, y: 60), Point2(x: 0, y: 60),
        ])
        XCTAssertThrowsError(try slotted.offset(10.0))
    }

    func testZeroAreaBoundaryHasNoOffsetDirection() throws {
        let bowtie = try PatternBoundary(points: [
            Point2(x: 0, y: 0), Point2(x: 100, y: 100),
            Point2(x: 100, y: 0), Point2(x: 0, y: 100),
        ])
        XCTAssertEqual(bowtie.winding, .degenerate)
        XCTAssertThrowsError(try bowtie.offset(10.0)) { error in
            XCTAssertEqual(error as? GeometryError, .indeterminateWinding)
        }
    }

    // MARK: - PatternPiece seam allowance

    func testNegativeSeamAllowanceIsRejected() throws {
        XCTAssertThrowsError(
            try PatternPiece(
                name: "Front Bodice",
                boundary: squareBoundary(side: 200),
                seamAllowanceMM: -1000.0
            )
        ) { error in
            XCTAssertEqual(error as? PatternError, .invalidSeamAllowance(valueMM: -1000.0))
        }
    }

    func testValidSeamAllowanceIsAcceptedAndCutBoundaryOffsets() throws {
        var piece = try PatternPiece(name: "Front Bodice", boundary: squareBoundary(side: 200))
        try piece.setSeamAllowanceMM(15.0)
        XCTAssertEqual(piece.seamAllowanceMM, 15.0)
        let cut = try piece.cutBoundary()
        XCTAssertGreaterThan(cut.perimeter, piece.boundary.perimeter)
    }

    // MARK: - JSON round-trips, matching the Rust engine's wire format

    func testPatternBoundaryRoundTripsThroughJSONMatchingRustShape() throws {
        let original = try squareBoundary(side: 10)
        let data = try JSONEncoder().encode(original)
        // The Rust engine's PatternBoundary serializes as a bare array via
        // #[serde(try_from = "Vec<Point2>", into = "Vec<Point2>")] — this
        // mirror must produce the same wire shape, not a wrapped object.
        let json = try XCTUnwrap(String(data: data, encoding: .utf8))
        XCTAssertTrue(json.hasPrefix("["), "expected a bare array, got \(json)")

        let restored = try JSONDecoder().decode(PatternBoundary.self, from: data)
        XCTAssertEqual(restored, original)
    }

    func testDecodingInvalidPatternBoundaryJSONFails() {
        let json = "[{\"x\":0,\"y\":0},{\"x\":50,\"y\":0}]".data(using: .utf8)!
        XCTAssertThrowsError(try JSONDecoder().decode(PatternBoundary.self, from: json))
    }

    func testMaterialDrapeSerializesLowercaseMatchingRust() throws {
        // Rust's Drape/Rigidity serialize with #[serde(rename_all = "snake_case")]
        // — Swift's String-backed raw values already match by construction,
        // this just pins that down as a regression test.
        let data = try JSONEncoder().encode(Drape.liquid)
        XCTAssertEqual(String(data: data, encoding: .utf8), "\"liquid\"")
    }
}
