//! Does a piece need to cache its flattened boundary?
//!
//! §3.6 declined a `#[serde(skip)]` boundary cache on `PatternPiece` on the
//! grounds that it is unmeasured optimisation, and that a cache is a second
//! source of truth for where cloth gets cut. This is where that call either
//! holds or does not.
//!
//! The previous wave measured **one** piece, in `patal-geometry`'s
//! `drag_loop` bench, and concluded the per-frame budget was comfortable. That
//! is the right answer to the question it asked — *can a drag recompute the
//! piece being dragged?* — and the wrong evidence for this one, because a
//! document-wide call touches every piece and a drag dirties exactly one.
//!
//! ## Why this bench lives in `patal-pattern`
//!
//! The execution plan put it in `engine/crates/geometry/benches/drag_loop.rs`.
//! That cannot work as written: `PatternPiece` and `Project` are this crate's
//! types and `patal-pattern` already depends on `patal-geometry`, so measuring
//! them from the geometry bench would require `patal-geometry` to
//! dev-depend on `patal-pattern`. Cargo permits that cycle, but it inverts the
//! single dependency direction the crate split exists to enforce, and it drags
//! the whole pattern layer into the kernel's bench build. Corrected here rather
//! than followed off the cliff; the plan's Task 11 entry records the deviation.
//!
//! ## The two calls, and why they are not the same measurement
//!
//! | Call | Work per piece |
//! |---|---|
//! | `PatternPiece::cut_boundary` | `flatten_for_offset` → `offset` (which runs `first_self_intersection` inside it) |
//! | `Project::total_perimeter_mm` | plain `flatten` → `perimeter` |
//!
//! Only the first pays for the O(n²) self-intersection test. A cache on the
//! flattened boundary would serve both, so both belong here — but if the
//! decision goes wrong it will go wrong on `cut_boundary`, not on perimeter.
//!
//! Both are swept across piece counts rather than measured at a single size.
//! One data point cannot distinguish linear from quadratic scaling, and the
//! whole question is what happens as a document grows.
//!
//! Budget, as everywhere in this repo: a 120Hz frame, **8.33ms**.
//!
//! ```sh
//! cmd //c 'scripts\cargo.bat bench --package patal-pattern'
//! ```
//!
//! # What the numbers said
//!
//! Measured 2026-08-17, release build, x86_64-pc-windows-msvc. All three
//! groups come from **one run**, because this machine is not stable enough
//! across runs for absolute figures to be compared between them (see the
//! caveat below). Within a run the comparisons are sound.
//!
//! Per piece, `cut_boundary`, against the 8330µs frame:
//!
//! | tolerance | per piece | share of frame |
//! |---|---|---|
//! | 0.5mm | 10.6µs | 0.13% |
//! | 0.1mm | 27.4µs | 0.33% |
//! | **0.01mm (default)** | **183µs** | **2.2%** |
//! | 0.001mm | 1.32ms | 15.9% |
//!
//! Document-wide, at the default 0.01mm:
//!
//! | pieces | `total_perimeter_mm` | all `cut_boundary` |
//! |---|---|---|
//! | 1 | 12.9µs | 185µs |
//! | 10 | 128µs | 1.90ms |
//! | **50** | **651µs — 7.8%** | **8.77ms — 105%** |
//! | 200 | 2.74ms — 33% | *not measured; linear* |
//!
//! Both scale **linearly** in piece count — 1:10:50:200 pieces gives
//! 1:10:50:213 cost. There is no cliff, so these extrapolate.
//!
//! ## The decision
//!
//! **The no-cache call holds, for the interaction it was made for.** A drag
//! dirties one piece, and one piece at the default tolerance is 2.2% of a
//! frame. Adding a `#[serde(skip)]` cache — a second source of truth for where
//! cloth gets cut — to reclaim 183µs on the one piece being dragged is not a
//! trade worth making.
//!
//! **But it holds with a constraint that is worth more than the decision.**
//! Recomputing every cut line in a 50-piece document costs **8.77ms, which is
//! more than a whole 120Hz frame.** That is not a cache problem — a full
//! recompute happens on export, on load, or on a global change, and 8.77ms
//! once is imperceptible. It is a *rendering* constraint: **the canvas must
//! never recompute all cut lines per frame.** Only dirty pieces may recompute.
//! Build a canvas that redraws every cut line on pan or zoom and this document
//! size drops below 120Hz, with no cache able to save it once the geometry is
//! genuinely dirty.
//!
//! `total_perimeter_mm` at 50 pieces is 651µs — 7.8% of a frame, which is
//! above the "few percent" the plan set as the tripwire. Recorded as such
//! rather than waved through. It is not a per-frame call — it is a document
//! statistic shown in a panel and recomputed on edit — so the practical
//! answer is unchanged, but the tripwire did fire and the checklist carries
//! the follow-up.
//!
//! Note which of these numbers the plan asked for: only `total_perimeter_mm`.
//! That is the *cheap* path — plain `flatten`, no offset, no O(n²)
//! self-intersection test — and on its own it would have come in at 7.8% and
//! been argued through. The 105% figure exists because this bench also
//! measures the expensive path, which is the one a cache would actually serve.
//!
//! ## Caveat: this machine does not reproduce the 2026-08-12 figures
//!
//! `patal-geometry`'s `drag_loop` header records 88µs for the equivalent
//! flatten→offset path at 0.01mm. Re-run on this machine on 2026-08-17 it
//! measures **429µs** — and every tolerance is 2.3–7× slower than recorded,
//! which is the signature of different hardware or thermal state rather than
//! a code regression in any one function.
//!
//! Two consequences. First, **do not compare a figure here against a figure
//! there**; only within-run comparisons mean anything. Second, `drag_loop`'s
//! conclusion survives its own re-measurement anyway — 429µs is still only
//! 5.2% of a frame — so nothing built on it needs revisiting. But that table
//! should name the machine it was taken on before it is quoted again.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use patal_geometry::{EdgeSegment, Point2, SeamPath};
use patal_pattern::{PatternPiece, Project};

/// A bodice front with four cubics: the same shape `drag_loop` measures, so
/// the two benches are comparable rather than each inventing a piece.
///
/// Deliberately a realistic garment panel and not a synthetic n-gon. A scooped
/// neckline, a shoulder seam, an armhole scye with the S-curve that makes
/// flattening non-trivial, a shaped side seam and a curved hem.
fn bodice_front() -> SeamPath {
    let start = Point2::new(0.0, 0.0);
    SeamPath::closed(
        start,
        vec![
            EdgeSegment::Cubic {
                c1: Point2::new(15.0, -30.0),
                c2: Point2::new(50.0, -22.0),
                to: Point2::new(75.0, 10.0),
            },
            EdgeSegment::Line {
                to: Point2::new(140.0, 30.0),
            },
            EdgeSegment::Cubic {
                c1: Point2::new(175.0, -30.0),
                c2: Point2::new(140.0, -130.0),
                to: Point2::new(160.0, -200.0),
            },
            EdgeSegment::Cubic {
                c1: Point2::new(140.0, -280.0),
                c2: Point2::new(165.0, -350.0),
                to: Point2::new(150.0, -420.0),
            },
            EdgeSegment::Cubic {
                c1: Point2::new(100.0, -430.0),
                c2: Point2::new(40.0, -425.0),
                to: Point2::new(0.0, -420.0),
            },
            EdgeSegment::Line { to: start },
        ],
    )
    .expect("bodice front closes")
}

/// A project of `n` identical bodice fronts at the default tolerance.
///
/// Identical on purpose: the question is how cost scales with piece *count*,
/// so varying the shapes would confound the one variable being swept.
fn project_of(n: usize) -> Project {
    let mut project = Project::new("Scale Test");
    let outline = bodice_front();
    for i in 0..n {
        project.add_piece(PatternPiece::new(format!("Piece {i}"), outline.clone()));
    }
    project
}

/// Per-piece cost of the expensive path, across the tolerance range.
///
/// This is the number a cache would be buying back on a single piece.
fn piece_cut_boundary(c: &mut Criterion) {
    let mut group = c.benchmark_group("piece_cut_boundary");
    let piece = PatternPiece::new("Bodice Front", bodice_front());

    // 0.01 is the shipped default; 0.4mm is roughly what an industrial cutter
    // can execute, so the two coarse settings bracket what anyone would
    // sensibly ask for and 0.001 is included to show the cliff is far away.
    for tolerance in [0.5f64, 0.1, 0.01, 0.001] {
        group.bench_with_input(
            BenchmarkId::from_parameter(tolerance),
            &tolerance,
            |b, &tolerance| {
                b.iter(|| black_box(piece.cut_boundary(black_box(tolerance)).unwrap()));
            },
        );
    }
    group.finish();
}

/// The call §3.6 named: every piece flattened, with no cache behind it.
///
/// Swept 1 → 200 so the scaling is visible. 50 is the plan's stated case; 200
/// is past any realistic garment so a surprise has somewhere to show up.
fn document_total_perimeter(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_total_perimeter");

    for pieces in [1usize, 10, 50, 200] {
        let project = project_of(pieces);
        group.bench_with_input(BenchmarkId::from_parameter(pieces), &pieces, |b, _| {
            b.iter(|| black_box(project.total_perimeter_mm().unwrap()))
        });
    }
    group.finish();
}

/// The worst honest case: every cut line in the document recomputed.
///
/// `total_perimeter_mm` is the call the plan named, but it takes the *cheap*
/// path — plain `flatten`, no offset, no self-intersection test. If a boundary
/// cache is ever justified it will be justified here, so measuring only
/// perimeter would let the decision through on the easier of the two numbers.
fn document_all_cut_boundaries(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_all_cut_boundaries");

    for pieces in [1usize, 10, 50] {
        let project = project_of(pieces);
        group.bench_with_input(BenchmarkId::from_parameter(pieces), &pieces, |b, _| {
            b.iter(|| {
                let mut total = 0.0;
                for piece in &project.pieces {
                    total += black_box(project.cut_boundary(piece).unwrap()).perimeter();
                }
                black_box(total)
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    piece_cut_boundary,
    document_total_perimeter,
    document_all_cut_boundaries
);
criterion_main!(benches);
