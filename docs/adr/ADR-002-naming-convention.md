---
id: ADR-002
title: Naming Convention — Pātāl / Patal
status: accepted
date: 2026-08-07
tags: [adr, naming, branding]
---

# ADR-002 — Naming Convention

## Status
**Accepted** — 2026-08-07. Permanent. Applies to all future work.

## Decision

Two forms of the project name. Both use **initial capital only** — never all-caps,
never all-lowercase in prose.

### `Pātāl` — display form
Used wherever Unicode diacritics render correctly:

- App UI, splash, about screen
- Marketing copy, website body text
- Vault notes and documentation
- Git commit messages and PR titles
- App Store display name (the field accepts Unicode)

### `Patal` — ASCII form
Used wherever diacritics break, get mangled, or are disallowed:

| Context | Value |
|---|---|
| Domain names | `satex25.co` (the actual site) — any future `patal.*` domain takes the ASCII form |
| Bundle identifier | `co.satex25.patal` (Target 1, Apple) — Target 2 differs, see below |
| Cargo crate names | `patal-geometry`, `patal-materials`, `patal-pattern`, `patal-ffi` |
| Git repository | `patal` |
| Swift module | `Patal` |
| File and folder paths in code | `Patal/` |
| Environment variables | `PATAL_*` (caps permitted here only) |
| CI job names | `patal-*` |

## Rationale
Diacritics are correct for the brand but hostile to toolchains. Cargo crate names are
restricted to ASCII alphanumerics, `-` and `_`. Bundle identifiers and domains are
ASCII-only by specification. Non-ASCII path components have historically caused
failures in Xcode build scripts and shell tooling.

Splitting the name resolves this without compromising the brand: users see `Pātāl`,
machines see `Patal`.

## Consequences
- Any new identifier defaults to the ASCII form unless it is user-visible text.
- The display form is never used in a path, identifier, or config key.
- Casing is fixed: `Pātāl` and `Patal`. Deviations are a bug.

## Correction 2026-08-12 — crate names

This table previously listed the crates as `patal-core` and `patal-ffi`. There has
never been a `patal-core`; the workspace splits into `patal-geometry`,
`patal-materials`, `patal-pattern` and `patal-ffi`. The table above is corrected. The
rule it illustrates — ASCII form for anything a toolchain touches — was always applied
correctly in the code; only the example was wrong.

Note also that the GitHub repository is `satex25/Patal` with a capital P, while this
ADR specifies `patal` lowercase for a git repository. That is a live inconsistency,
left alone deliberately: renaming churns every URL for no benefit, and GitHub
repository names are case-insensitive for resolution. It is recorded here so nobody
later "fixes" it believing the ADR endorsed the capital.

## Note on the current vault path
The Obsidian vault currently lives at `Desktop\Pātāl\Pātāl\` — display form in a
filesystem path. This is tolerable because the vault holds notes, not build inputs.
The **code** workspace must use the ASCII form.

**Resolved 2026-08-07:** the code now lives at `C:\Users\User\patal\` (git
repository, branch `master`) — fully ASCII, not even nested under the vault's
diacritic parent, satisfying this rule with margin to spare.

**Superseded 2026-08-13:** there is no separate vault path any more. The notes were
folded into `docs/` in this repository, so the only location on disk is the ASCII
`C:\Users\User\patal\`. `C:\Users\User\Desktop\patal` is a directory junction pointing
at it, not a second copy — also ASCII. The rule now has nothing left to constrain: no
path in the project carries a diacritic. Display form survives only in prose and in
the product name itself, which is exactly where this ADR wanted it.

## Bundle identifiers — resolved 2026-08-07

This ADR previously specified `com.patal.app` while the shipped
`apps/desktop/src-tauri/tauri.conf.json` used `co.satex25.patal`, and parked the
contradiction as an open question. Resolved in favour of the `co.satex25.*` scheme
for both targets:

- **Reverse-DNS should be rooted in a domain you control.** `satex25.co` is owned and
  is already the download/site host (`README.md`, and the Tauri `homepage` field).
  `patal.com` and `patal.app` are **not** owned, so `com.patal.app` would have claimed
  a namespace belonging to someone else.
- `com.patal.app` was also degenerate on its own terms — it parses as organisation
  `patal.com` plus product `app`, where the product name is literally "app".
- The Tauri config was already correct under the chosen scheme, so the code barely moved.

**Accepted cost:** the identifier permanently reads `satex25`, so Pātāl is branded as a
satex25 product at the OS level. If Pātāl is ever spun out under its own domain, the
Apple identifier cannot follow it (see permanence below).

### The two targets must not share one identifier
ADR-001 ships **macOS builds from both targets** (Target 1: iOS/iPadOS/macOS;
Target 2: Windows + macOS). Two distinct macOS apps carrying the same bundle
identifier collide in LaunchServices — app registration, preference domains, and
container paths all key off it. They are therefore distinguished at the leaf:

| Target | Identifier |
|---|---|
| Target 1 — Apple native (active) | `co.satex25.patal` |
| Target 2 — Tauri desktop (frozen) | `co.satex25.patal.desktop` |

Target 1 takes the unqualified form because its identifier is the permanent one.

### Target 1's identifier is permanent
An Apple bundle identifier cannot be changed after the first App Store submission — it
is the app's identity in App Store Connect for the life of the app. Nothing has shipped
under either identifier yet (Target 1 has no Xcode project at all; Target 2 has never
been signed), which is precisely why this was free to settle now. Set
`PRODUCT_BUNDLE_IDENTIFIER = co.satex25.patal` when the Xcode project is first created,
and treat it as immutable from that point.
