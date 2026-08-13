---
title: Rename (Patruin → Pātāl/Patal) + Relocate Implementation Plan
date: 2026-08-07
status: draft
tags: [plan, rename, relocate]
---

# Rename + Relocate Implementation Plan

> **Completed.** The rename shipped, the code lives at `C:\Users\User\patal`, and the
> naming rule is ADR-002 — see [the ADR index](../adr/README.md). Kept as the record of
> how it was done. Current state: [status](../status.md). Audit of what was renamed:
> [the inherited-codebase analysis](../analysis/inherited-codebase.md).

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the inherited codebase out of its double-nested zip-extract location into a clean workspace, execute the Patruin→Pātāl/Patal rename across all 26 affected files, and leave the repo building/testing green under git version control (currently none exists).

**Architecture:** No structural/behavioral changes — this is a rename + relocation only. Sequence: relocate the pristine tree first and commit it as a baseline (so the rename is a clean, reviewable diff on top), then rename bottom-up through the dependency graph (geometry/materials → pattern → ffi → the two front ends → prose docs), verifying build/test green after each layer.

**Tech Stack:** Rust 1.97.1 (cargo workspace), Swift package (no Xcode/macOS available in this environment — Swift changes are static-only, unverified by `swift build` until Mac access exists), Tauri 2 + npm/Node 24.15.0 (below the package.json floor of 24.18.1 — pre-existing gap, not introduced by this plan, not fixed here).

## Global Constraints (ADR-002, accepted, permanent)
- Two forms of the name, initial-capital only, never all-caps/all-lowercase in prose:
  - **`Pātāl`** (display, Unicode) — UI text, window/nav titles, App Store display name, docs prose, commit messages.
  - **`Patal`** (ASCII) — domains, bundle/package identifiers, Cargo crate names, git repo name, Swift module/target names, file/folder paths in code, env vars (`PATAL_*`).
- Any new identifier defaults to ASCII unless it is literally user-visible text.
- Per the inherited-codebase audit (`03 Codebase/Inherited Codebase — Full Analysis.md` §7): 124 occurrences of `patruin`/`Patruin` across these 26 files — confirmed by fresh grep, count matches exactly. Not a find-and-replace: lowercase/hyphenated code tokens are unambiguous, but every capitalized "Patruin" needs a per-site display-vs-identifier call (enumerated in Task 3/4/5 below).
- Both `engine/Cargo.lock` and `apps/desktop/src-tauri/Cargo.lock` must be **regenerated**, never hand-edited. Same for `apps/desktop/package-lock.json`.
- Reference repos at `Desktop\Pātāl\reference\` are untouched by this plan — read-only, not part of the rename.

---

## File Structure

Target repo root after Task 1: **`C:\Users\User\patal\`** (lowercase — matches ADR-002's "Git repository: `patal`" row and this user's existing sibling-project convention: `C:\Users\User\mc4`, `mc4-rust`, `mc5`, `mc8`). Confirmed empty/unused before this plan runs.

No new files are created; existing files are edited or moved:
- `engine/crates/{geometry,materials,pattern,ffi}/Cargo.toml` — package `name`, path-dependency names
- `engine/crates/{materials,pattern,ffi}/src/lib.rs` — `use` paths, doc-comment references, `.patruin`→`.patal` mentions (`geometry/src/lib.rs` has **zero** occurrences — confirmed by grep, not touched)
- `apps/native/Sources/PatruinKit/` → `apps/native/Sources/PatalKit/` (dir rename) — `Package.swift`, `Models/Geometry.swift`, `Models/Material.swift`, `Models/Project.swift`, `Views/ContentView.swift`
- `apps/native/Tests/PatruinKitTests/` → `apps/native/Tests/PatalKitTests/` (dir rename)
- `apps/native/README.md`
- `apps/desktop/src-tauri/{Cargo.toml,src/main.rs,src/lib.rs,tauri.conf.json}`
- `apps/desktop/{package.json,index.html,README.md}`
- `README.md`, `docs/memorandum.md` (root — prose rewrite, not mechanical)
- `engine/Cargo.lock`, `apps/desktop/src-tauri/Cargo.lock`, `apps/desktop/package-lock.json` — deleted + regenerated, not edited

---

### Task 1: Relocate to a clean workspace and establish a git baseline

**Files:** none edited — pure filesystem move + git init.

- [ ] **Step 1: Move the inherited tree**

```bash
mkdir -p "/c/Users/User/patal"
cp -r "/c/Users/User/Desktop/Pātāl/Patal-main/Patal-main/." "/c/Users/User/patal/"
```

- [ ] **Step 2: Verify nothing was lost (file count must match)**

```bash
diff <(cd "/c/Users/User/Desktop/Pātāl/Patal-main/Patal-main" && find . -type f | sort) \
     <(cd "/c/Users/User/patal" && find . -type f | sort)
```
Expected: no output (identical file lists).

- [ ] **Step 3: Remove the now-redundant double-nested source, once Step 2 is clean**

```bash
rm -rf "/c/Users/User/Desktop/Pātāl/Patal-main"
```

- [ ] **Step 4: Confirm the move didn't break relative build paths**

```bash
cd "/c/Users/User/patal/engine" && cargo test --workspace
```
Expected: `47 passed; 0 failed` across the four crates (same as the audit's baseline — this is the pristine, not-yet-renamed code, so it must still be exactly 47).

- [ ] **Step 5: git init + baseline commit**

```bash
cd "/c/Users/User/patal"
git init
git add -A
git commit -m "Import inherited Patruin codebase (pre-rename baseline)"
```

No prior commit exists anywhere for this code — this is the first one, and it captures the exact inherited state before any renaming, so the rename itself is a clean, reviewable diff on top.

- [ ] **Step 6: Update the vault's location note**

The vault's `02 Setup/Reference Repositories.md` and the audit doc both say the source lives at `Desktop\Pātāl\Patal-main\Patal-main\`. That path no longer exists after Step 3. Add a one-line pointer at the top of `03 Codebase/Inherited Codebase — Full Analysis.md`:

```markdown
> **Relocated 2026-08-07:** this audit was performed at `Desktop\Pātāl\Patal-main\Patal-main\`,
> which no longer exists — the code now lives at `C:\Users\User\patal\` (git-initialized,
> see the rename plan in `04 Plans/`). Paths below are historical.
```

---

### Task 2: Rename the Rust engine crates

**Files:**
- Modify: `engine/crates/geometry/Cargo.toml`
- Modify: `engine/crates/materials/Cargo.toml`, `engine/crates/materials/src/lib.rs`
- Modify: `engine/crates/pattern/Cargo.toml`, `engine/crates/pattern/src/lib.rs`
- Modify: `engine/crates/ffi/Cargo.toml`, `engine/crates/ffi/src/lib.rs`
- Regenerate: `engine/Cargo.lock`

**Interfaces:** crate names only change; no public API signatures change. After this task, downstream consumers (Task 3 Swift doc-comments, Task 4 Tauri desktop) reference `patal_geometry`, `patal_materials`, `patal_pattern`, `patal-ffi` instead of the `patruin_*`/`patruin-*` forms.

- [ ] **Step 1: Baseline grep (confirm starting count)**

```bash
cd "/c/Users/User/patal" && grep -ril "patruin" engine/ | sort
```
Expected: exactly these 7 files —
```
engine/Cargo.lock
engine/crates/ffi/Cargo.toml
engine/crates/ffi/src/lib.rs
engine/crates/materials/Cargo.toml
engine/crates/materials/src/lib.rs
engine/crates/pattern/Cargo.toml
engine/crates/pattern/src/lib.rs
```

- [ ] **Step 2: Package names — one line each, in every crate's `Cargo.toml`**

| File | Old | New |
|---|---|---|
| `crates/geometry/Cargo.toml` | `name = "patruin-geometry"` | `name = "patal-geometry"` |
| `crates/materials/Cargo.toml` | `name = "patruin-materials"` | `name = "patal-materials"` |
| `crates/pattern/Cargo.toml` | `name = "patruin-pattern"` | `name = "patal-pattern"` |
| `crates/ffi/Cargo.toml` | `name = "patruin-ffi"` | `name = "patal-ffi"` |

- [ ] **Step 3: Path-dependency names**

`crates/pattern/Cargo.toml`:
```toml
[dependencies]
patruin-geometry = { path = "../geometry" }
patruin-materials = { path = "../materials" }
```
→
```toml
[dependencies]
patal-geometry = { path = "../geometry" }
patal-materials = { path = "../materials" }
```

`crates/ffi/Cargo.toml`:
```toml
[dependencies]
patruin-geometry = { path = "../geometry" }
patruin-materials = { path = "../materials" }
patruin-pattern = { path = "../pattern" }
```
→
```toml
[dependencies]
patal-geometry = { path = "../geometry" }
patal-materials = { path = "../materials" }
patal-pattern = { path = "../pattern" }
```

- [ ] **Step 4: `use` paths and doc comments — `crates/materials/src/lib.rs`**

Three occurrences, all mechanical token swaps (`patruin_` → `patal_`, and the Swift module name inside a Rust doc comment):
1. Line 10: `matching PatruinKit's Swift` → `matching PatalKit's Swift`
2. Line 77: `` a real invariant (`patruin_geometry::PatternBoundary`) needs `` → `` a real invariant (`patal_geometry::PatternBoundary`) needs ``
3. Line 170 (test comment): `// PatruinKit's Swift` → `// PatalKit's Swift`

- [ ] **Step 5: `use` paths and doc/test comments — `crates/pattern/src/lib.rs`**

```rust
use patruin_geometry::{GeometryError, PatternBoundary};
use patruin_materials::Material;
```
→
```rust
use patal_geometry::{GeometryError, PatternBoundary};
use patal_materials::Material;
```

Doc comment (`PatternPiece`, ~line 71): `` a `.patruin` file edited by hand `` → `` a `.patal` file edited by hand ``

In `#[cfg(test)] mod tests`:
```rust
use patruin_geometry::Point2;
use patruin_materials::Material;
```
→
```rust
use patal_geometry::Point2;
use patal_materials::Material;
```

Test comment (`deserializing_negative_seam_allowance_is_rejected`): `// .patruin file load a piece` → `// .patal file load a piece`

- [ ] **Step 6: `use` paths and doc comments — `crates/ffi/src/lib.rs`**

```rust
//! Domain crates (`patruin-geometry`, `patruin-materials`, `patruin-pattern`)
```
→
```rust
//! Domain crates (`patal-geometry`, `patal-materials`, `patal-pattern`)
```

```rust
use patruin_geometry::GeometryError;
```
→
```rust
use patal_geometry::GeometryError;
```

All remaining occurrences in this file are the fully-qualified path `patruin_geometry::Point2` / `patruin_geometry::PatternBoundary`, appearing 6 times (1 doc comment + 5 in code: the `From<Point>` impl, its body, the reverse `From` impl, its body, and `boundary_from`'s signature + body). Every one becomes `patal_geometry::` — same identifier, no other change:

```rust
/// An FFI-safe 2D point. Converts to/from `patruin_geometry::Point2`.
...
impl From<Point> for patruin_geometry::Point2 {
    fn from(p: Point) -> Self {
        patruin_geometry::Point2::new(p.x, p.y)
    }
}

impl From<patruin_geometry::Point2> for Point {
    fn from(p: patruin_geometry::Point2) -> Self {
        Point { x: p.x, y: p.y }
    }
}

fn boundary_from(points: Vec<Point>) -> Result<patruin_geometry::PatternBoundary, EngineError> {
    Ok(patruin_geometry::PatternBoundary::new(
```
→ (`patruin_geometry` → `patal_geometry` in all 7 spots, nothing else changes)

- [ ] **Step 7: Regenerate the lockfile**

```bash
cd "/c/Users/User/patal/engine"
rm Cargo.lock
cargo test --workspace
```
Expected: `Cargo.lock` is recreated and **47 passed; 0 failed** — same count as Task 1 Step 4, proving the rename changed no behavior.

- [ ] **Step 8: Quality gates (the original README's stated bar)**

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```
Expected: both clean.

- [ ] **Step 9: Confirm zero leftover occurrences, then commit**

```bash
grep -ril "patruin" engine/
```
Expected: no output.

```bash
git add engine/
git commit -m "Rename Rust engine crates: patruin-* -> patal-*"
```

---

### Task 3: Rename the Swift package (PatruinKit → PatalKit)

**Files:**
- Move: `apps/native/Sources/PatruinKit/` → `apps/native/Sources/PatalKit/`
- Move: `apps/native/Tests/PatruinKitTests/` → `apps/native/Tests/PatalKitTests/`
- Modify: `apps/native/Package.swift`, `apps/native/README.md`
- Modify: `apps/native/Sources/PatalKit/Models/Geometry.swift`, `Material.swift`, `Project.swift`, `Views/ContentView.swift`
- Modify: `apps/native/Tests/PatalKitTests/PatalKitTests.swift`

**Not verifiable by build in this environment:** no `swift` toolchain on Windows (confirmed: `swift --version` → command not found). This task is static-edit-only; `swift build`/`swift test` must be run once Mac access exists (Toolchain checklist Phase 2). Step 8 substitutes a grep-based check.

- [ ] **Step 1: Rename the directories first (git tracks the move)**

```bash
cd "/c/Users/User/patal"
git mv apps/native/Sources/PatruinKit apps/native/Sources/PatalKit
git mv apps/native/Tests/PatruinKitTests apps/native/Tests/PatalKitTests
```

- [ ] **Step 2: `Package.swift`**

```swift
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
```
→
```swift
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
```
(Package `name` is the ASCII identifier form per ADR-002 — ships in `Package.resolved`/Xcode project references, not user-visible.)

- [ ] **Step 3: `Sources/PatalKit/Models/Geometry.swift` — one display-text occurrence**

```swift
/// favor of the real uniffi bindings, this is the seam-allowance math for
/// three of Patruin's four target platforms — see the README.
```
→
```swift
/// favor of the real uniffi bindings, this is the seam-allowance math for
/// three of Pātāl's four target platforms — see the README.
```
(Product name mentioned in prose describing the brand → display form. All other "mirrors `patruin_geometry::...`"-style doc comments in this file already say `patruin_geometry` in **Rust** path syntax — apply the same `patruin_` → `patal_` token swap as Task 2, no display/identifier judgment needed since it's referencing the Rust module path, not the product name.)

- [ ] **Step 4: `Sources/PatalKit/Views/ContentView.swift` — the one runtime-visible string**

```swift
.navigationTitle("Patruin")
```
→
```swift
.navigationTitle("Pātāl")
```
(This is literally the nav-bar title shown on screen — display form, unambiguous.)

- [ ] **Step 5: `Tests/PatalKitTests/PatalKitTests.swift`**

```swift
import XCTest
@testable import PatruinKit

final class PatruinKitTests: XCTestCase {
```
→
```swift
import XCTest
@testable import PatalKit

final class PatalKitTests: XCTestCase {
```

- [ ] **Step 6: `apps/native/README.md`** — mixed display/identifier, five sites

| Old | New | Why |
|---|---|---|
| `# Patruin — Native App (iPhone, iPad, Mac)` | `# Pātāl — Native App (iPhone, iPad, Mac)` | doc title, prose |
| `` `PatruinKit` is a Swift package `` | `` `PatalKit` is a Swift package `` | code identifier |
| `product name **Patruin**, interface` | `product name **Pātāl**, interface` | ADR-002: App Store/Xcode display name field is Unicode-accepting |
| `Tests/PatruinKitTests` (path reference) | `Tests/PatalKitTests` | path |
| every other `PatruinKit` (models list, drift-risk paragraph) | `PatalKit` | code identifier |

The embedded code sample also changes (identifiers, not display — this is Swift source, not UI text):
```swift
import SwiftUI
import PatruinKit

@main
struct PatruinApp: App {
```
→
```swift
import SwiftUI
import PatalKit

@main
struct PatalApp: App {
```

- [ ] **Step 7: Confirm zero leftover occurrences**

```bash
grep -ril "patruin" apps/native/
```
Expected: no output.

- [ ] **Step 8: Static consistency check (build substitute — no Swift toolchain here)**

```bash
grep -rn "PatalKit\|patal_geometry\|patal_materials\|patal_pattern" apps/native/Sources apps/native/Tests | wc -l
```
Sanity-check the count is non-zero and every file compiles *by eye*: every `import PatalKit` has a matching `Sources/PatalKit/` target, every doc-comment `patal_geometry::X` names a symbol that still exists (unchanged from Task 2, only the prefix moved). Flag in the commit message that `swift build`/`swift test` are still pending on Mac access.

- [ ] **Step 9: Commit**

```bash
git add apps/native/
git commit -m "Rename Swift package: PatruinKit -> PatalKit (unverified by swift build - no macOS in this environment)"
```

---

### Task 4: Rename the Tauri desktop app

**Files:**
- Modify: `apps/desktop/src-tauri/Cargo.toml`, `src/main.rs`, `src/lib.rs`, `tauri.conf.json`
- Modify: `apps/desktop/package.json`, `index.html`, `README.md`
- Regenerate: `apps/desktop/src-tauri/Cargo.lock`, `apps/desktop/package-lock.json`

**Note:** Node here is v24.15.0; `package.json` declares `"engines": { "node": ">=24.18.1" }`. That gap predates this plan and isn't this task's to fix — `npm install` will likely just warn, not fail, but flag it if it does.

- [ ] **Step 1: `src-tauri/Cargo.toml`**

```toml
[package]
name = "patruin-desktop"
version = "0.1.0"
description = "Patruin desktop app (Windows/Mac downloadable build)"
```
→
```toml
[package]
name = "patal-desktop"
version = "0.1.0"
description = "Patal desktop app (Windows/Mac downloadable build)"
```

```toml
[lib]
name = "patruin_desktop_lib"
```
→
```toml
[lib]
name = "patal_desktop_lib"
```

```toml
[dependencies]
patruin-geometry = { path = "../../../engine/crates/geometry" }
patruin-pattern = { path = "../../../engine/crates/pattern" }
```
→
```toml
[dependencies]
patal-geometry = { path = "../../../engine/crates/geometry" }
patal-pattern = { path = "../../../engine/crates/pattern" }
```

- [ ] **Step 2: `src-tauri/src/main.rs`**

```rust
fn main() {
    patruin_desktop_lib::run()
}
```
→
```rust
fn main() {
    patal_desktop_lib::run()
}
```

- [ ] **Step 3: `src-tauri/src/lib.rs`**

```rust
use patruin_geometry::{PatternBoundary, Point2};
use patruin_pattern::{PatternPiece, Project};
```
→
```rust
use patal_geometry::{PatternBoundary, Point2};
use patal_pattern::{PatternPiece, Project};
```

- [ ] **Step 4: `src-tauri/tauri.conf.json`**

```json
"productName": "Patruin",
"version": "0.1.0",
"identifier": "co.satex25.patruin",
```
→
```json
"productName": "Patal",
"version": "0.1.0",
"identifier": "co.satex25.patal",
```
`productName` stays ASCII, not `Pātāl` — unlike the SwiftUI nav title, this field also seeds installer/artifact filenames on Windows packaging, the exact toolchain-hostility case ADR-002's rationale warns about, even though the ADR's table doesn't list Tauri by name. `identifier` keeps the existing `co.satex25.*` reverse-domain scheme (matches the real `satex25.co` domain used elsewhere in this same file) rather than switching to the `com.patal.desktop` form the audit doc floated — smaller, lower-risk change, and Target 2 is frozen per ADR-001 so this isn't imminently load-bearing. **Flag for the user:** ADR-002's own table says bundle identifiers take the form `com.patal.app` — that row and this choice disagree on the scheme; cheap to change later since Target 2 is frozen, easy to revisit if you want the two consistent now instead.

> **Resolved 2026-08-07 (after this plan executed).** The flag above was actioned: the
> `co.satex25.*` scheme won and ADR-002's table was corrected to match, on the grounds
> that `satex25.co` is owned while `patal.com` / `patal.app` are not. The identifier
> shipped by this plan was then further qualified to `co.satex25.patal.desktop`, because
> Target 1 and Target 2 both produce macOS builds and cannot share one bundle
> identifier. Final values live in ADR-002 — treat that as authoritative over this line.

```json
"windows": [
  {
    "title": "Patruin",
```
→
```json
"windows": [
  {
    "title": "Pātāl",
```
(Runtime window title, pure UI text — display form, unambiguous.)

- [ ] **Step 5: `apps/desktop/package.json`**

```json
{
  "name": "patruin-desktop",
```
→
```json
{
  "name": "patal-desktop",
```

- [ ] **Step 6: `apps/desktop/index.html`**

```html
<title>Patruin</title>
```
→
```html
<title>Pātāl</title>
```

```html
<h1 class="text-2xl font-semibold tracking-tight">Patruin</h1>
```
→
```html
<h1 class="text-2xl font-semibold tracking-tight">Pātāl</h1>
```
(Both are literally rendered to the user — display form.)

- [ ] **Step 7: `apps/desktop/README.md`**

```markdown
# Patruin — Desktop (Windows / Mac downloadable build)
```
→
```markdown
# Pātāl — Desktop (Windows / Mac downloadable build)
```

```markdown
which builds a `Project` from `patruin-pattern` and returns its perimeter —
```
→
```markdown
which builds a `Project` from `patal-pattern` and returns its perimeter —
```

- [ ] **Step 8: Regenerate lockfiles**

```bash
cd "/c/Users/User/patal/apps/desktop/src-tauri"
rm Cargo.lock
cargo check
```
Expected: resolves and compiles clean (this crate has no tests of its own — `engine_demo_perimeter_mm` is exercised manually via the running app, not `cargo test`).

```bash
cd "/c/Users/User/patal/apps/desktop"
rm package-lock.json
npm install
```
Expected: installs clean (watch for the Node engine-version warning noted above — expected, not a new failure).

- [ ] **Step 9: Confirm zero leftover occurrences, then commit**

```bash
grep -ril "patruin" apps/desktop/
```
Expected: no output.

```bash
git add apps/desktop/
git commit -m "Rename Tauri desktop app: patruin-desktop -> patal-desktop"
```

---

### Task 5: Rewrite the root README.md

**Files:** Modify `README.md` (repo root).

This file's references are 90% mechanical (crate names, directory tree labels, repo URL) but the opening line is prose that becomes false after the rename (the Irish-etymology framing doesn't apply to "Pātāl"), so this task hand-authors the new file rather than token-swapping. **Flag for the user:** the paragraph below is a drafted replacement for the naming rationale — it states the real meaning of *Pātāl* (a Sanskrit/Hindi term for the netherworld, one of the seven realms below earth in Hindu cosmology) since nothing in the vault records why this name was chosen; confirm or rewrite it before this file is treated as final, since it's a branding-intent claim, not a mechanical fact.

- [ ] **Step 1: Title and opening line**

```markdown
# Patruin

Patruin — the modern Irish word for patterns (patrún). 

A professional garment pattern creation platform: from idea to production-ready pattern in
one workspace, across iPhone, iPad, Mac, and Windows.

- Repo: [github.com/satex25/patruin](https://github.com/satex25/patruin)
- Website / downloads: [satex25.co](https://satex25.co)
```
→
```markdown
# Pātāl

Pātāl (पाताल) — in Hindu cosmology, the netherworld: one of the seven realms
beneath the earth, vast and richly structured, built downward from a surface
few ever see. Formerly named *Patruin* (Irish "patrún," pattern); renamed
2026-08-07 — see `01 Architecture/Decisions/ADR-002 Naming Convention.md`.

A professional garment pattern creation platform: from idea to production-ready pattern in
one workspace, across iPhone, iPad, Mac, and Windows.

- Repo: [github.com/satex25/patal](https://github.com/satex25/patal)
- Website / downloads: [satex25.co](https://satex25.co)
```

- [ ] **Step 2: Architecture tree**

```markdown
patruin/
├── engine/                 Rust workspace — platform-agnostic core
│   └── crates/
│       ├── geometry/       patruin-geometry  — Point2, PatternBoundary, seam-allowance offset
│       ├── materials/      patruin-materials — Material, MaterialLibrary
│       ├── pattern/        patruin-pattern   — PatternPiece, Project, measurements
│       └── ffi/            patruin-ffi       — uniffi bindings exposed to Swift
```
→
```markdown
patal/
├── engine/                 Rust workspace — platform-agnostic core
│   └── crates/
│       ├── geometry/       patal-geometry  — Point2, PatternBoundary, seam-allowance offset
│       ├── materials/      patal-materials — Material, MaterialLibrary
│       ├── pattern/        patal-pattern   — PatternPiece, Project, measurements
│       └── ffi/            patal-ffi       — uniffi bindings exposed to Swift
```

- [ ] **Step 3: Remaining body mentions — mechanical token table**

Apply `patruin-ffi`→`patal-ffi`, `patruin-geometry`→`patal-geometry`, `patruin-pattern`→`patal-pattern`, `patruin-materials`→`patal-materials`, `PatruinKit`→`PatalKit` everywhere else in the file (the "Why this split" paragraph, the "Status" section's `engine/`/`patruin-ffi`/`apps/native`/`apps/desktop` bullets, and the "Swift mirror is duplicated" paragraph). No further display-text judgment calls remain in this file — the only two prose/display sites were the title and opening line (Step 1).

- [ ] **Step 4: Confirm zero leftover occurrences, then commit**

```bash
grep -i "patruin" README.md
```
Expected: no output.

```bash
git add README.md
git commit -m "Rewrite root README for the Patal rename"
```

---

### Task 6: Rewrite docs/memorandum.md

**Files:** Modify `docs/memorandum.md`.

Only the "Project Name" section names the product; every other section (Mission, Vision, Core Philosophy, etc.) is name-agnostic prose that needs no edits — confirmed by grep, `patruin`/`Patruin` appears exactly twice in this file, both in the "Project Name" section.

**Flag for the user:** same caveat as Task 5 — the replacement below states the Pātāl/netherworld meaning as the new name's rationale. Confirm or rewrite before treating this founding document as final.

- [ ] **Step 1: Replace the Project Name section**

```markdown
## Project Name

Patruin

Patrúin (plural of patrún) is the modern Irish word for patterns. The name
represents far more than sewing patterns—it embodies the idea of a blueprint,
a design language, a model for creation, and the foundation from which new
ideas emerge. It reflects both centuries of craftsmanship and the limitless
possibilities of future innovation.
```
→
```markdown
## Project Name

Pātāl

Pātāl (पाताल) is the netherworld of Hindu cosmology — one of seven realms
built downward beneath the visible earth, each with its own depth and
structure. The name represents far more than a play on "pattern"—it embodies
the idea of a workspace built in layers, with real structure beneath what a
designer first sees on the surface: an idea becomes a silhouette, a
silhouette becomes construction geometry, construction geometry becomes a
production-ready pattern. It reflects both the depth of craftsmanship the
platform is built on and the layered, structured nature of garment
engineering itself.

Formerly named *Patruin* (Irish "patrún," pattern) — renamed 2026-08-07; see
`01 Architecture/Decisions/ADR-002 Naming Convention.md` for the naming rules
this document now follows (`Pātāl` in prose, `Patal` in code/identifiers).
```

- [ ] **Step 2: Confirm zero leftover occurrences, then commit**

```bash
grep -i "patruin" docs/memorandum.md
```
Expected: no output.

```bash
git add docs/memorandum.md
git commit -m "Rewrite memorandum's Project Name section for the Patal rename"
```

---

### Task 7: Final repo-wide verification sweep

**Files:** none — verification only, no edits expected. If this task finds anything, it means an earlier task's grep check was wrong, and this is a bug in a prior task, not new work.

- [ ] **Step 1: Repo-wide grep, every file, case-insensitive**

```bash
cd "/c/Users/User/patal"
grep -ril "patruin" . --exclude-dir=.git --exclude-dir=target --exclude-dir=node_modules --exclude-dir=dist
```
Expected: no output. If anything appears, it was missed by Tasks 2–6 — fix it there conceptually (identifier vs. display, per this plan's rules above) before moving on.

- [ ] **Step 2: Full engine test + lint pass**

```bash
cd "/c/Users/User/patal/engine"
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```
Expected: 47 passed; 0 failed, clippy clean, fmt clean.

- [ ] **Step 3: Desktop compiles**

```bash
cd "/c/Users/User/patal/apps/desktop/src-tauri"
cargo check
```
Expected: clean.

- [ ] **Step 4: Record what's still unverified**

Add a line to the vault's `02 Setup/Toolchain Install Checklist.md` Open section:
```markdown
- [ ] Swift rename (Task 3 of the rename plan) — static-edit only, `swift build`/`swift test`
      not run. Verify for real once Mac access exists.
```

No commit needed for this task unless Step 1 turns up something to fix — in that case, fix and commit under whichever Task 2–6 the file belongs to, then re-run Step 1.

---

## Self-Review

**Spec coverage:** all 26 files from the audit's occurrence count are covered across Tasks 2–6 (7 in Task 2, 7 in Task 3, 6 in Task 4 — Cargo.lock/package-lock.json regenerated not hand-edited, so counted but not itemized — 1 in Task 5, 1 in Task 6 — total 22 hand-edited + 4 lockfiles-by-regeneration accounts for all 26). Relocation and git-baseline (not in the audit's count, since it's new work this plan adds) is Task 1. Every ADR-002 naming rule (display vs. ASCII) is applied per-site with reasoning shown, not asserted blind.

**Placeholder scan:** every step shows literal before/after content or an exact command; the two spots needing a genuine content decision (README/memorandum naming rationale) are explicitly flagged as drafts for sign-off rather than silently invented, per this plan's own instructions — that is a call for the user, not a gap in the plan.

**Type/name consistency:** `patal_geometry`/`patal_materials`/`patal_pattern`/`patal-ffi` (Task 2) are the exact tokens Task 3's doc-comments and Task 4's `use` statements are written against — checked matching across all four tasks.
