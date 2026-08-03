import XCTest
@testable import PatruinKit

final class PatruinKitTests: XCTestCase {
    private func squareBoundary(side: Double) -> PatternBoundary {
        PatternBoundary(points: [
            Point2(x: 0, y: 0),
            Point2(x: side, y: 0),
            Point2(x: side, y: side),
            Point2(x: 0, y: side),
        ])
    }

    func testPerimeterOfSquare() {
        XCTAssertEqual(squareBoundary(side: 10).perimeter, 40.0, accuracy: 1e-9)
    }

    func testProjectTotalPerimeterSumsPieces() {
        let piece = PatternPiece(name: "Panel", boundary: squareBoundary(side: 10))
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
}
