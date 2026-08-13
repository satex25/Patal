---
title: Reference Repositories
date: 2026-08-07
tags: [setup, reference, github]
---

# Reference Repositories

Part of [the docs index](../README.md) · setup context for [toolchain setup](toolchain.md) · current state
in [status](../status.md).

Cloned to `reference/` at the repository root, and **git-ignored there** — see the
`/reference/` rule in `.gitignore`. Each clone carries its own `.git`, so committing
them would record broken gitlinks rather than files, and ~83 MB of third-party source
does not belong in this project's history.

They are read-only prior art, not dependencies. Nothing in the build reads them.

| Repo | Size | Commit read | Upstream | Why |
|---|---|---|---|---|
| `swift-bridge` | 8.2M | `e527dc7` | `chinedufn/swift-bridge` | Rust↔Swift FFI. Supports async, generics, and high-level types. Bridge candidate A. |
| `uniffi-rs` | 8.7M | `bc6a335` | `mozilla/uniffi-rs` | Mozilla's binding generator. Ships in Firefox. Bridge candidate B. |
| `cargo-swift` | 1.2M | `e11f075` | `antoniusnaumann/cargo-swift` | Wraps UniFFI — turns a Rust crate into a Swift Package with xcframework in one command. |
| `ferrostar` | 46M | `c163960` | `stadiamaps/ferrostar` | **Most valuable.** A shipped, production cross-platform navigation SDK: Rust core + SwiftUI + xcframework packaging. Closest existing analogue to our architecture. |
| `XcodeBuildMCP` | 19M | `e6ef59b` | `getsentry/XcodeBuildMCP` | MCP server for agent-driven Xcode builds, simulator control, log capture, UI automation. Needed on the Mac. |

Total: ~83M

## Re-creating them

Because they are ignored, a fresh clone of Pātāl will not have them:

```bash
mkdir -p reference && cd reference
git clone --depth 1 --single-branch https://github.com/chinedufn/swift-bridge.git
git clone --depth 1 --single-branch https://github.com/mozilla/uniffi-rs.git
git clone --depth 1 --single-branch https://github.com/antoniusnaumann/cargo-swift.git
git clone --depth 1 --single-branch https://github.com/stadiamaps/ferrostar.git
git clone --depth 1 --single-branch https://github.com/getsentry/XcodeBuildMCP.git
```

A fresh shallow clone lands newer than the commits above; those record what was
actually read.

## Reading priority
1. **ferrostar** — study its repo layout, xcframework build script, and how the Swift
   package wraps the Rust core. This is the template.
2. **cargo-swift** — the packaging path we will most likely adopt.
3. **swift-bridge** vs **uniffi-rs** — compare once the data shapes are known (ADR-001 open item).
