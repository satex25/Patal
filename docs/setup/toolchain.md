---
title: Toolchain Install Checklist
status: in-progress
date: 2026-08-07
tags: [setup, toolchain, environment]
---

# Toolchain Install Checklist

Part of [the docs index](../README.md) · see also [reference repositories](reference-repositories.md) · current state in [status](../status.md).

> **The repo now documents its own prerequisites.** `README.md` in
> `C:\Users\User\patal` carries the build prerequisites, and `CONTRIBUTING.md` carries
> the full gate — including the Git Bash `link.exe` shadowing trap and the committed
> `scripts/cargo.bat` that works around it. Prefer those; they are version-controlled
> and this note is not. What stays useful here is the machine-setup narrative below.

> Nothing here installs *into* the Pātāl vault. The vault holds notes.
> Tools install to the operating system. This note is the record of what to run.

## Platform reality

| Layer | Windows | macOS |
|---|---|---|
| Rust core | ✅ full | ✅ full |
| Rust unit tests | ✅ full | ✅ full |
| Swift / SwiftUI | ❌ | ✅ |
| Metal | ❌ | ✅ |
| Xcode build + simulator | ❌ | ✅ |
| App Store submission | ❌ | ✅ |

**Consequence:** the entire Rust core can be written and tested on Windows.
The Apple layer is blocked until Mac access exists.

---

## Phase 1 — Windows, available now

### 1.1 Rust toolchain
```powershell
winget install --id Rustlang.Rustup -e
```
Verify:
```powershell
rustc --version
cargo --version
```

### 1.2 Core Rust quality gates
```powershell
rustup component add clippy rustfmt
cargo install cargo-nextest
cargo install cargo-deny
```

### 1.3 Rust MCP servers (Claude Code connectivity)
```powershell
cargo install rust-docs-mcp
claude mcp add rust-docs -- rust-docs-mcp
```
```powershell
cargo install rust-analyzer-mcp
claude mcp add rust-analyzer -- rust-analyzer-mcp
```

### 1.4 GitHub MCP
```powershell
claude mcp add --transport http github https://api.githubcopilot.com/mcp/
```

---

## Phase 2 — macOS, blocked until Mac access

### 2.1 Xcode
Install from the Mac App Store, then:
```bash
xcode-select --install
sudo xcodebuild -license accept
```

### 2.2 Rust Apple targets
```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin x86_64-apple-darwin
```

### 2.3 Rust → Swift bridge
```bash
cargo install cargo-swift
```
Alternative, if the FFI boundary proves hot:
```bash
cargo install swift-bridge-cli
```

### 2.4 XcodeBuildMCP — highest-value tool in the list
Lets the agent build, run the simulator, read logs, screenshot, and drive UI
automation directly. Without it, code is written blind and errors get pasted back
by hand.
```bash
claude mcp add XcodeBuildMCP -- npx -y xcodebuildmcp@latest
```

### 2.5 Tuist — Xcode project as code
Prevents `.pbxproj` XML corruption, which is the most common way agents break
Xcode projects.
```bash
curl -Ls https://install.tuist.io | bash
```

### 2.6 Swift quality gates
```bash
brew install swiftlint swiftformat
```

---

## Verification — 2026-08-07

Confirmed by terminal output on the Windows machine:

| Tool | Version | Status |
|---|---|---|
| rustc | 1.97.1 | ✅ |
| cargo | 1.97.1 | ✅ |
| clippy | 0.1.97 | ✅ |
| rustfmt | 1.9.0-stable | ✅ |
| cargo-nextest | 0.9.143 | ✅ |
| cargo-deny | 0.20.2 | ✅ |
| rust-docs-mcp | — | ✅ connected |
| rust-analyzer-mcp | v0.2.0 | ✅ connected |
| github MCP | — | ❌ auth failure |

Host triple: `x86_64-pc-windows-msvc`

### Issue 1 — MCP scope bound to system32
Servers were added from `C:\WINDOWS\system32`, so they registered at *local project*
scope tied to that directory. They will not resolve from the real project folder.

Fix:
```powershell
claude mcp add --scope user rust-docs -- rust-docs-mcp
claude mcp add --scope user rust-analyzer -- rust-analyzer-mcp
```

### Issue 2 — GitHub MCP auth
`Incompatible auth server: does not support dynamic client registration`.
The remote endpoint requires an OAuth flow the client cannot complete.
Use a personal access token instead (scopes: `repo`, `read:org`):
```powershell
claude mcp remove github
claude mcp add --scope user --transport http github https://api.githubcopilot.com/mcp/ --header "Authorization: Bearer YOUR_PAT_HERE"
```

### Also present (pre-existing, unrelated)
Notion (needs auth), Google Drive (connected), chrome-devtools (connected).

## Open
- [x] Mac access confirmed — available, but not within the first few days.
      Phase 2 deferred accordingly. Phase 1 + Rust core proceed on Windows.
- [x] Phase 1 — Rust toolchain complete and verified
- [ ] Phase 1 — MCP scope fix + GitHub auth outstanding
- [ ] Swift rename (Task 3 of the rename plan) — static-edit only, `swift build`/`swift test`
      not run. Verify for real once Mac access exists.
- [ ] Phase 2 complete
