# Patruin — Desktop (Windows / Mac downloadable build)

Tauri app: Rust backend (`src-tauri/`, linking the shared engine crates in
`../../engine` directly as path dependencies) + a Tailwind-styled web
frontend (`src/`). This is the build distributed from satex25.co, primarily
for Windows — Mac users get the native SwiftUI app in `../native` instead,
but this Tauri build also runs on Mac if ever needed.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Commands

```sh
npm install
npm run tauri dev    # run the app locally
npm run build         # typecheck + build the frontend
npm run tauri build   # produce a distributable installer
```

## Current state

`index.html` / `src/main.ts` are a placeholder screen with one button that
calls the `engine_demo_perimeter_mm` Tauri command (`src-tauri/src/lib.rs`),
which builds a `Project` from `patruin-pattern` and returns its perimeter —
proof the desktop shell is wired to the real engine, not real product UI
yet.
