//! Does the cut path fit inside a frame?
//!
//! The architectural answer to `self_intersects` being O(n²) has been "cache
//! cut boundaries, recompute on edit, never per frame". That is true right
//! up until the interaction that matters most: **dragging a control point is
//! a stream of edits at frame rate**, so during a drag "on edit" and "per
//! frame" are the same thing. If the real path costs more than a frame, the
//! answer is not a faster algorithm, it is a different interaction — a
//! coarse preview flatten while the mouse is down, refined on release — and
//! that decision has to be made before `SeamPath`'s shape is fixed, not
//! after a canvas is built on top of it.
//!
//! So this measures the whole path a drag actually triggers:
//!
//! ```text
//!   flatten_for_offset  ->  offset  ->  self_intersects (inside offset)
//! ```
//!
//! against a 120Hz budget of 8.33ms. Run it with:
//!
//! ```sh
//! cmd //c 'scripts\cargo.bat bench -p patal-geometry'
//! ```
//!
//! Measure before optimising. A sweep-line self-intersection test is a real
//! option, and it is the wrong thing to build if the numbers say the budget
//! is comfortable.
//!
//! # What the numbers said
//!
//! Measured 2026-08-12, release build, x86_64-pc-windows-msvc, on the
//! four-cubic bodice front below. Whole path per frame, against 8330µs:
//!
//! | tolerance | vertices | per frame | share of budget |
//! |---|---|---|---|
//! | 0.5mm | 43 | 5.4µs | 0.06% |
//! | 0.1mm | 89 | 13.3µs | 0.16% |
//! | 0.01mm | 247 | 88µs | 1.1% |
//! | 0.001mm | 821 | 640µs | 7.7% |
//!
//! **The concern does not survive contact with the measurement.** Even at
//! 0.001mm — forty times finer than an industrial cutter can execute, and
//! finer than any reason to ask for — a full recompute costs under a tenth
//! of a frame. At the 0.01mm setting that actually matters it is about one
//! percent.
//!
//! So the coarse-preview-flatten-during-drag strategy should **not** be
//! built. It would add a two-representation state machine, and a class of
//! bug where the preview and the committed geometry disagree, to solve a
//! problem that is roughly ninety times away from existing.
//!
//! The O(n²) scaling is real and confirmed — `self_intersects` goes 57ns →
//! 6.9µs → 664µs across n = 10 → 100 → 1000, and it dominates `offset`,
//! which it runs inside. Extrapolating quadratically, the frame budget is
//! reached somewhere near 3000 vertices in a single piece. A garment piece
//! at manufacturing tolerance is around 250. Revisit if pieces ever get an
//! order of magnitude more complex, or if many pieces must recompute in one
//! frame; a drag today only dirties the piece being dragged.
//!
//! One thing this does **not** measure, and it should be said rather than
//! implied: the FFI round trip. These are in-process Rust timings. ADR-001's
//! rule that the render loop never crosses FFI per frame is untouched by
//! this result, and a per-frame `flatten → offset` called *through* uniffi
//! would still violate it regardless of how cheap the geometry is.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use patal_geometry::{EdgeSegment, PatternBoundary, Point2, SeamPath};

/// A four-cubic neckline on a bodice front: the shape the benchmark exists
/// for. Two curves sweeping from shoulder to centre front, closed by
/// straight shoulder and centre-front seams.
fn neckline() -> SeamPath {
    let start = Point2::new(0.0, 0.0);
    SeamPath::closed(
        start,
        vec![
            // Scooped neckline, centre front out to the shoulder point.
            // Concave: it removes material, which is what a scoop is.
            EdgeSegment::Cubic {
                c1: Point2::new(15.0, -30.0),
                c2: Point2::new(50.0, -22.0),
                to: Point2::new(75.0, 10.0),
            },
            // Shoulder seam.
            EdgeSegment::Line {
                to: Point2::new(140.0, 30.0),
            },
            // Armhole scye: the S-curve that makes this worth benchmarking.
            EdgeSegment::Cubic {
                c1: Point2::new(175.0, -30.0),
                c2: Point2::new(140.0, -130.0),
                to: Point2::new(160.0, -200.0),
            },
            // Side seam with waist shaping.
            EdgeSegment::Cubic {
                c1: Point2::new(140.0, -280.0),
                c2: Point2::new(165.0, -350.0),
                to: Point2::new(150.0, -420.0),
            },
            // Hem, gently curved.
            EdgeSegment::Cubic {
                c1: Point2::new(100.0, -430.0),
                c2: Point2::new(40.0, -425.0),
                to: Point2::new(0.0, -420.0),
            },
            // Centre front, straight, back to the start.
            EdgeSegment::Line { to: start },
        ],
    )
    .expect("neckline closes")
}

/// A regular n-gon, for isolating how each kernel operation scales.
fn polygon(n: usize) -> PatternBoundary {
    let points = (0..n)
        .map(|i| {
            let theta = std::f64::consts::TAU * i as f64 / n as f64;
            Point2::new(200.0 * theta.cos(), 200.0 * theta.sin())
        })
        .collect();
    PatternBoundary::new(points).expect("a regular polygon is valid")
}

/// The measurement the interaction design depends on.
fn drag_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("drag_frame");
    let path = neckline();

    // 0.4mm is roughly what an industrial cutter can execute, so anything
    // finer is spent on screen fidelity rather than on the garment. If the
    // tight end blows the budget and the coarse end does not, that is the
    // preview-during-drag strategy handing itself to us.
    for tolerance in [0.5f64, 0.1, 0.01, 0.001] {
        group.bench_with_input(
            BenchmarkId::new("flatten_offset_check", tolerance),
            &tolerance,
            |b, &tolerance| {
                b.iter(|| {
                    let flat = path
                        .flatten_for_offset(black_box(tolerance), black_box(10.0))
                        .unwrap();
                    // `offset` runs `first_self_intersection` internally, so
                    // this is the whole per-frame cost, not a piece of it.
                    black_box(flat.offset(black_box(10.0)).unwrap())
                });
            },
        );
    }
    group.finish();
}

/// How many vertices each tolerance actually produces — the number that
/// makes the O(n²) figure concrete.
fn flatten_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("flatten");
    let path = neckline();

    for tolerance in [0.5f64, 0.1, 0.01, 0.001] {
        let n = path.flatten(tolerance).unwrap().points().len();
        group.bench_with_input(
            BenchmarkId::new(format!("tolerance_{tolerance}_n{n}"), tolerance),
            &tolerance,
            |b, &tolerance| b.iter(|| black_box(path.flatten(black_box(tolerance)).unwrap())),
        );
    }
    group.finish();
}

fn kernel_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("kernel");

    for n in [10usize, 100, 1000] {
        let boundary = polygon(n);

        group.bench_with_input(BenchmarkId::new("offset", n), &n, |b, _| {
            b.iter(|| black_box(boundary.offset(black_box(5.0)).unwrap()))
        });

        group.bench_with_input(BenchmarkId::new("self_intersects", n), &n, |b, _| {
            b.iter(|| black_box(boundary.first_self_intersection()))
        });

        group.bench_with_input(BenchmarkId::new("perimeter", n), &n, |b, _| {
            b.iter(|| black_box(boundary.perimeter()))
        });
    }
    group.finish();
}

criterion_group!(benches, drag_frame, flatten_only, kernel_scaling);
criterion_main!(benches);
