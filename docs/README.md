# Pātāl — documentation

Garment pattern CAD. From idea to production-ready pattern in one workspace, across
iPhone, iPad, Mac and Windows.

**Start here:** [status.md](status.md) — where the work actually is, updated at the end
of each session. If it disagrees with anything else in this tree, it wins.

## The one rule

From the header of `engine/crates/geometry/src/lib.rs`, and it outranks everything else
in this documentation:

> Every operation here is either correct or loud. A pattern piece that is silently
> wrong is worse than one that refuses to compute: the first gets cut out of cloth,
> the second gets fixed.

## The tree

| Path | What lives there |
|---|---|
| [status.md](status.md) | Current state, what is in flight, what is next |
| [roadmap.md](roadmap.md) | The pillars that are not built yet, and why that is fine |
| [memorandum.md](memorandum.md) | The founding vision document |
| [adr/](adr/) | Architecture Decision Records — rules the code must obey |
| [setup/](setup/) | Toolchain installation, reference repositories, and [printing at true scale](setup/printing.md) — the last of which is an operator runbook, not a developer note |
| [analysis/](analysis/) | Audits of the codebase as found, and of the domain it models — including [the pattern primitive census](analysis/pattern-primitives.md) and [the incumbent persistence probe](analysis/incumbent-persistence-probe.md) that supplies its evidence |
| [plans/](plans/) | Dated session plans and execution blueprints |

## Why the decisions are in here

The ADRs began in an Obsidian vault outside version control. The README and the
memorandum both cite them normatively — "see ADR-002" — while nobody cloning the repo
could obtain them. A rule you cannot read is not a rule.

They are engineering artifacts: they constrain code, they are cited by code
documentation, and they should be reviewable in the same diff as the change that obeys
or amends them. The same reasoning now applies to this whole tree, which is why the
vault's thinking notes were folded in rather than kept beside the repo.

**Do not keep a second copy anywhere.** Two versions of a decision is worse than none —
the failure mode an ADR exists to prevent is someone acting on a rule that has quietly
changed.

## Reading this as an Obsidian vault

Open `docs/` itself as the vault. Its `.obsidian/` workspace config is committed-adjacent
but git-ignored, so layout and graph settings stay local to your machine while the notes
stay versioned. Links are plain relative Markdown rather than `[[wikilinks]]` so they
resolve on GitHub as well as in Obsidian.

## What Pātāl is not

Not a drawing program with a garment theme. The distinction that matters is that a
pattern is *a system of relationships* — change a bust measurement and the pieces that
depend on it should follow. That solver is the largest unbuilt thing here, and
everything in [roadmap.md](roadmap.md) is downstream of taking it seriously.
