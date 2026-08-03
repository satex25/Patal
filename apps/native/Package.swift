// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Patruin",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(name: "PatruinKit", targets: ["PatruinKit"]),
    ],
    targets: [
        .target(name: "PatruinKit"),
        .testTarget(name: "PatruinKitTests", dependencies: ["PatruinKit"]),
    ]
)
