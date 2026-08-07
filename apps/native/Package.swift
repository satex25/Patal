// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Patal",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(name: "PatalKit", targets: ["PatalKit"]),
    ],
    targets: [
        .target(name: "PatalKit"),
        .testTarget(name: "PatalKitTests", dependencies: ["PatalKit"]),
    ]
)
