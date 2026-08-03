# Patruin — Native App (iPhone, iPad, Mac)

`PatruinKit` is a Swift package holding the shared models and SwiftUI views
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
2. `File > New > Project > App`, product name **Patruin**, interface
   **SwiftUI**, targeting iOS, and check "Mac" under the destination
   platforms (a SwiftUI multiplatform app — not Mac Catalyst).
3. Save it inside `apps/native/` alongside this `Package.swift`.
4. Add this directory as a local Swift package dependency
   (`File > Add Package Dependencies > Add Local...`, select
   `apps/native`), and link `PatruinKit` to the app target.
5. Replace the generated `ContentView` with:

   ```swift
   import SwiftUI
   import PatruinKit

   @main
   struct PatruinApp: App {
       var body: some Scene {
           WindowGroup {
               ContentView()
           }
       }
   }
   ```

## Current state

`PatruinKit`'s models (`Point2`, `PatternBoundary`, `Material`,
`PatternPiece`, `Project`) are hand-written Swift mirrors of the Rust engine
types in `../../engine`. They exist so the UI has something real to build
against today. The next milestone is replacing them with the uniffi-generated
Swift bindings from `patruin-ffi`, packaged as an XCFramework — at that point
these files collapse into thin view-model wrappers around the generated
types instead of parallel implementations.
