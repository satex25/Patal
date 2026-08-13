# Pātāl — engineering harness (Tauri)

**This is not a shipping target.** ADR-001 rejected Tauri for the product on
native-feel grounds and that decision stands. ADR-005 records why this app is
unfrozen anyway.

The short version: this is the only thing in the repo that runs on the
machine Pātāl is developed on. There is no macOS toolchain here, so
`apps/native` cannot even be compiled locally; CI is its only build. And this
app links the engine crates directly, as path dependencies, with no FFI
boundary in between — so it can put a seam allowance on screen today, with
nothing to generate and nothing to bridge.

That makes it worth about ten lines of glue and worth deleting the moment the
native app can do the same job.

Tauri app: Rust backend (`src-tauri/`, linking the engine crates in
`../../engine`) + a Tailwind-styled web frontend (`src/`).

## What it does

A bodice front — four cubics and two straight seams, the same shape the
drag-loop benchmark measures — drawn live with two sliders:

- **Flattening tolerance.** Watch vertex count and per-frame cost move as you
  tighten it. The panel shows the cost as a share of a 120Hz frame, against
  the same 8333µs budget `benches/drag_loop.rs` reports.
- **Seam allowance**, positive or negative. Push it past what the curvature
  can give and the engine refuses; the refusal appears next to the shape that
  caused it, in the engine's own words. That is the message a designer will
  eventually see, and this is where a human reads it first.

There is also a button that writes a real `.patal` file and reads it back,
reporting whether it round-tripped and whether the piece's material reference
survived. The engine has no persistence layer — that was deliberately cut —
so the file handling lives here, in disposable code, where it exercises the
document format without committing the engine to an API for it.

## Commands

```sh
npm install
npm run tauri dev     # run the harness
npm run build         # typecheck + build the frontend
cargo test            # in src-tauri/ — the commands are tested headlessly
```

On Windows with Git Bash, route cargo through the wrapper (see the root
README's prerequisites):

```sh
PATAL_CARGO_DIR='C:\path\to\patal\apps\desktop\src-tauri' cmd //c 'scripts\cargo.bat test'
```

## Rules for this directory

1. **Nothing here is a product decision.** If a question about how Pātāl
   should behave gets answered in this app, the answer belongs in the engine
   or in an ADR, not in `src/main.ts`.
2. **Never reimplement engine logic.** The harness calls `patal-geometry`; it
   does not compute geometry. The Swift package once carried a second copy of
   the offset kernel and it was deleted for exactly this reason.
3. **Show the engine's errors verbatim.** Paraphrasing them starts a second
   error vocabulary that has to be kept in sync with the first.
4. It stays a CI compile gate regardless — `cargo clippy --all-targets
   -D warnings` runs against it on every push, so a change to a domain type's
   shape breaks here loudly rather than silently rotting.
