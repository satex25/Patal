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
against today. `PatternBoundary.offset` and `PatternPiece`'s seam-allowance
validation are ported from the Rust engine's implementation directly — same
mitre limit, same bevel joins, same self-intersection and winding checks,
same errors thrown instead of a wrong-looking number — so this is no longer
a thin display-only mirror.

That does not make it a binding. It is still a second, hand-maintained
implementation that can drift from the Rust engine on the next change to
either side, and `PatternPiece`/`Project` still carry a `UUID` that Rust's
types have no counterpart for, so `PatternPiece`'s JSON shape is
Swift-to-Swift only (unlike `PatternBoundary`'s, which matches the Rust
engine's wire format exactly). The next milestone is still replacing all of
this with the uniffi-generated Swift bindings from `patal-ffi`, packaged
as an XCFramework — at that point these files collapse into thin
view-model wrappers around the generated types instead of parallel
implementations.

**A note on how the offset port was verified in this environment:** `swift
test` needs full Xcode for `XCTest`, which isn't installed here (see above).
The XCTest cases in `Tests/PatalKitTests` are written but couldn't be
executed locally as a result. Instead, every one of the Rust engine's own
numeric test cases — including the exact inputs that used to corrupt the
old Rust kernel with NaN, fling a vertex hundreds of millimetres off a
piece, or silently invert a winding — was ported into a throwaway
executable target, run with `swift run`, and checked against the Rust
engine's own output to six decimal places. All matched. The throwaway
target was then deleted; it never touched `Package.swift` in the committed
state. Once Xcode is installed, `swift test` should be run for real before
trusting this note over the actual test suite.
