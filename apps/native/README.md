# Pātāl — Native App (iPhone, iPad, Mac)

`PatalKit` is a Swift package holding the shared models and SwiftUI views
used across iPhone, iPad, and Mac — one native codebase for all three, per
the memorandum's platform goals.

It builds and (once Xcode is installed) tests standalone:

```sh
swift build
swift test
```

This directory is a Swift package, not yet an Xcode project — full Xcode
(not just the Command Line Tools) is needed to produce an actual `.app`
and to run on the iOS Simulator or physical devices. To wire it up:

1. Install Xcode from the App Store, then run it once so it finishes
   installing its additional components.
2. `File > New > Project > App`, product name **Patal**, interface
   **SwiftUI**, targeting iOS, and check "Mac" under the destination
   platforms (a SwiftUI multiplatform app — not Mac Catalyst).
3. Save it inside `apps/native/` alongside this `Package.swift`.
4. Add this directory as a local Swift package dependency
   (`File > Add Package Dependencies > Add Local...`, select
   `apps/native`), and link `PatalKit` to the app target.
5. Replace the generated `ContentView` with:

   ```swift
   import SwiftUI
   import PatalKit

   @main
   struct PatalApp: App {
       var body: some Scene {
           WindowGroup {
               ContentView()
           }
       }
   }
   ```

## Current state

`PatalKit`'s models (`Point2`, `PatternBoundary`, `Material`,
`PatternPiece`, `Project`) are hand-written Swift mirrors of the Rust engine
types in `../../engine`. They exist so the UI has something real to build
against today.

**This package holds no geometry, on purpose.** It carries
`PatternBoundary`'s *construction contract* — at least three finite points,
consecutive duplicates dropped, private storage behind a read-only
accessor, `Codable` matching the Rust engine's bare-point-array wire shape
— and its perimeter, a sum of distances with no error surface. It does not
offset, and it does not compute winding, signed area, or self-intersection.

It used to. A 368-line line-for-line port of the Rust offset kernel lived
here: same mitre limit, same bevel joins, same validity checks. That made
two independent implementations of the geometry that decides where cloth
gets cut, with nothing checking them against each other — and the failure
mode of drift is not a red test, it is a garment cut wrong. The alternative
to deleting it was a committed corpus of golden vectors asserted by both
test suites. That would have worked, and would also have taxed every future
change to the engine's error surface with a matching edit here, forever, to
pin code with a scheduled death date. Nothing depended on the port: there
is no Xcode project in this repo, and `cutBoundary()`'s only caller was its
own test suite. So it was deleted instead.

Seam-allowance geometry reaches this package through uniffi-generated
bindings from `patal-ffi`, packaged as an XCFramework, once a Mac is
available to build one. At that point these files become thin view-model
wrappers around generated types rather than parallel implementations.

The remaining divergence is the identity model: `PatternPiece` and `Project`
carry a `UUID` that Rust's types have no counterpart for, so
`PatternPiece`'s JSON shape is Swift-to-Swift only, unlike
`PatternBoundary`'s, which matches the Rust engine's wire format exactly.

**Nothing here has ever been compiled.** There is no macOS toolchain on the
machine this is developed on. CI's `native` job (`swift build` + `swift
test` on `macos-latest`) is the only verification this code has, and its
first green run will be the first evidence the package builds at all.
