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
    //
    // The offset, winding, signed-area and self-intersection tests that used
    // to sit below this section went with the code they covered. Seam
    // allowance is computed by `patal-geometry`, which has its own 30 tests
    // for exactly these cases; duplicating them here only proved that a
    // second implementation agreed with itself.

    func testDuplicateConsecutivePointsAreDropped() throws {
        // Mirrors the Rust engine's regression test for the bug this
        // normalization exists to prevent: a duplicate point used to divide
        // by a zero-length edge and poison the boundary with NaN.
        let boundary = try PatternBoundary(points: [
            Point2(x: 0, y: 0),
            Point2(x: 100, y: 0),
            Point2(x: 100, y: 0),
            Point2(x: 100, y: 200),
            Point2(x: 0, y: 200),
        ])
        XCTAssertEqual(boundary.points.count, 4)
    }

    func testClosingPointRepeatingTheFirstIsDropped() throws {
        let boundary = try PatternBoundary(points: [
            Point2(x: 0, y: 0),
            Point2(x: 10, y: 0),
            Point2(x: 10, y: 10),
            Point2(x: 0, y: 0),
        ])
        XCTAssertEqual(boundary.points.count, 3)
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

    func testValidSeamAllowanceIsStored() throws {
        var piece = try PatternPiece(name: "Front Bodice", boundary: squareBoundary(side: 200))
        try piece.setSeamAllowanceMM(15.0)
        XCTAssertEqual(piece.seamAllowanceMM, 15.0)
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
