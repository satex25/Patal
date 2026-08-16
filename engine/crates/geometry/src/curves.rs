//! Authored curve geometry: the shape a designer draws, above the polygon
//! kernel that decides where cloth is cut.
//!
//! Necklines, armholes, sleeve caps and hems are curves. [`PatternBoundary`]
//! is deliberately a straight-edged polygon and stays that way — teaching it
//! about curves would put curve-curve intersection on the cut path and
//! require rewriting `self_intersects`, `signed_area`, `winding` and
//! `offset` at once, all of which are load-bearing and all of which are
//! tested. So this is a second layer instead:
//!
//! ```text
//!   SeamPath  ──flatten(tolerance)──▶  PatternBoundary  ──offset──▶  cut line
//!   (authored)                          (manufactured)
//! ```
//!
//! That split is the industry norm — parametric authoring, discretized
//! manufacturing — and it is metrologically free at garment scale.
//! Industrial fabric cutting works to roughly 0.4mm. A flattening tolerance
//! of 0.01mm is about forty times finer than any cutter can execute and far
//! finer than cloth can hold, so flatten-then-offset is indistinguishable
//! from true curve offsetting *for this application*. The argument is
//! scale-dependent and would not survive being moved to optical tooling.
//!
//! `PatternBoundary`'s shape never changes as a result of any of this, so
//! the FFI signatures that take one keep working forever.

use serde::{Deserialize, Serialize};

use crate::{GeometryError, PatternBoundary, Point2};

/// How far subdivision will go before it stops splitting.
///
/// Flattening error falls by roughly 4x per level, so 24 levels is about
/// 2.8e14 — unreachable for any tolerance a garment implies. It exists so
/// that a pathological curve (a cusp, a tolerance of 1e-300) terminates
/// instead of exhausting the stack.
const MAX_SUBDIVISION_DEPTH: u8 = 24;

/// Samples per cubic when estimating curvature. Garment curves are smooth
/// and low-degree; 33 samples locate the maximum of a cubic's curvature
/// closely enough for a tolerance estimate, and the oracle test is what
/// actually holds the result to account.
const CURVATURE_SAMPLES: usize = 32;

/// Ceiling on how far [`SeamPath::flatten_for_offset`] will tighten.
///
/// Without it, a near-cusp curvature estimate drives the tolerance to zero
/// and subdivision runs to the depth limit producing millions of points. A
/// curvature that extreme means the requested offset is not representable
/// at all, which is a real failure — but it is one the kernel decides, and
/// it reports it properly as `OffsetSelfIntersects` or `OffsetCollapsed`
/// once the flattened boundary reaches `offset`. Clamping here keeps this
/// function total and leaves the loud failure where it can be judged.
const MAX_TOLERANCE_TIGHTENING: f64 = 1.0e4;

/// How close to its own start a path must land before [`SeamPath::closed`]
/// treats the difference as float noise and snaps it, rather than a gap the
/// designer left and wants an edge across.
///
/// Relative to the coordinates involved, so it means the same thing at
/// buttonhole and bolt-of-cloth scale. At 1e-12 the absolute threshold on a
/// 1000mm piece is a picometre — many orders below anything a cutter, a
/// fabric, or a designer can express, and many orders above the ~1e-16
/// noise that trigonometry leaves behind.
const CLOSURE_SNAP_RELATIVE: f64 = 1.0e-12;

/// One authored edge, starting wherever the previous one ended.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EdgeSegment {
    /// A straight run to `to`.
    Line { to: Point2 },
    /// A cubic Bézier to `to`, shaped by two control points.
    ///
    /// Cubic rather than quadratic because a cubic can express an S-curve
    /// in one segment — which is what a sleeve cap and a princess seam
    /// both are — and because every drawing tool a designer has used
    /// speaks cubics.
    Cubic { c1: Point2, c2: Point2, to: Point2 },
}

impl EdgeSegment {
    /// Where this segment ends.
    pub fn end(&self) -> Point2 {
        match self {
            Self::Line { to } => *to,
            Self::Cubic { to, .. } => *to,
        }
    }

    fn control_points(&self) -> impl Iterator<Item = Point2> + '_ {
        let points: Vec<Point2> = match self {
            Self::Line { to } => vec![*to],
            Self::Cubic { c1, c2, to } => vec![*c1, *c2, *to],
        };
        points.into_iter()
    }
}

/// How an edge meets the edge before it.
///
/// A *claim about intent*, which is why it is stored rather than re-derived:
/// two collinear handles might be coincidence, and a designer who wants
/// tangency preserved through an edit needs somewhere to say so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Join {
    /// No continuity claim. The *absence* of a claim, which is why it is the
    /// serde default: omitting the key cannot manufacture a claim the
    /// coordinates contradict.
    #[default]
    Corner,
    /// The tangents on either side of this join are parallel and same-signed.
    Smooth,
}

/// One authored edge: its geometry, and how it meets the edge before it.
///
/// # Why a struct around one field
///
/// `join` is the *first* per-edge attribute, not the only one. The primitive
/// census identifies three more that are attributes of an edge — per-edge
/// seam allowance (P-03), fold edges (P-05) and notch anchors (P-13) — and
/// per-edge allowance is a fold-in rather than a maybe, because a neckline is
/// finished at 6mm and a hem is turned at 40mm.
///
/// Each of those arriving as its own array parallel to the geometry adds a
/// `len ==` invariant and one more thing every edit that splits a segment
/// must keep in step. Four arrays is four chances to get it wrong, and the
/// fourth gets it wrong on the path that feeds the cut line. This struct is
/// what makes adding the second attribute a field rather than a schema
/// migration.
///
/// Do not flatten it back.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    geometry: EdgeSegment,
    #[serde(default)]
    join: Join,
}

impl Edge {
    pub fn new(geometry: EdgeSegment, join: Join) -> Self {
        Self { geometry, join }
    }

    /// An edge making no continuity claim — what every edge built through
    /// [`SeamPath::new`] or [`SeamPath::closed`] is.
    pub fn corner(geometry: EdgeSegment) -> Self {
        Self {
            geometry,
            join: Join::Corner,
        }
    }

    pub fn geometry(&self) -> EdgeSegment {
        self.geometry
    }

    pub fn join(&self) -> Join {
        self.join
    }

    /// Where this edge ends. Delegates to the geometry.
    pub fn end(&self) -> Point2 {
        self.geometry.end()
    }
}

/// The wire shape of a [`SeamPath`]: exactly its fields, so a document is
/// readable without knowing anything about this type's invariants.
#[derive(Serialize, Deserialize)]
struct SeamPathData {
    start: Point2,
    edges: Vec<Edge>,
}

/// A closed, authored outline: where it starts, and the edges that walk it
/// back around to that point.
///
/// # Invariants
///
/// Construction is the only way in, exactly as with [`PatternBoundary`]:
/// every control point is finite, there is at least one edge, and the last
/// edge ends precisely where the path started. `serde` routes through
/// [`SeamPath::with_joins`] via `try_from`, so a hand-edited file cannot
/// smuggle in an open or non-finite path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "SeamPathData", into = "SeamPathData")]
pub struct SeamPath {
    start: Point2,
    edges: Vec<Edge>,
}

impl From<SeamPath> for SeamPathData {
    fn from(path: SeamPath) -> Self {
        Self {
            start: path.start,
            edges: path.edges,
        }
    }
}

impl TryFrom<SeamPathData> for SeamPath {
    type Error = GeometryError;

    fn try_from(data: SeamPathData) -> Result<Self, Self::Error> {
        Self::with_joins(data.start, data.edges)
    }
}

impl SeamPath {
    /// Validates a closed path. The last segment must end exactly at
    /// `start`.
    ///
    /// Exact equality, and no silent repair. An auto-appended closing edge
    /// is geometry the caller did not draw, and this crate does not invent
    /// geometry — see [`SeamPath::closed`] for the case where you want that
    /// edge and want to say so.
    pub fn new(start: Point2, segments: Vec<EdgeSegment>) -> Result<Self, GeometryError> {
        Self::with_joins(start, segments.into_iter().map(Edge::corner).collect())
    }

    /// [`SeamPath::new`] for a path that makes continuity claims.
    ///
    /// `edges[i].join` describes the join *entering* `edges[i]`, so
    /// `edges[0].join` is the closure join at `start`. There is no parallel
    /// array to keep in step, so there is no off-by-one to get wrong.
    pub fn with_joins(start: Point2, edges: Vec<Edge>) -> Result<Self, GeometryError> {
        if !start.is_finite() {
            return Err(GeometryError::NonFiniteControlPoint { segment: 0 });
        }
        if edges.is_empty() {
            return Err(GeometryError::TooFewPoints { count: 1 });
        }

        for (index, edge) in edges.iter().enumerate() {
            if edge.geometry().control_points().any(|p| !p.is_finite()) {
                return Err(GeometryError::NonFiniteControlPoint { segment: index });
            }
        }

        let end = edges.last().expect("checked non-empty").end();
        if end != start {
            return Err(GeometryError::PathNotClosed { start, end });
        }

        Ok(Self { start, edges })
    }

    /// [`SeamPath::new`], but makes the path close instead of refusing.
    ///
    /// Two different situations arrive here and they need different
    /// answers, so this distinguishes them rather than treating every gap
    /// the same:
    ///
    /// * A **real** gap — the designer drew a neckline and expects the
    ///   shoulder seam to close the piece — gets a real `Line` appended,
    ///   visible to anything that inspects the path.
    /// * **Float noise** gets snapped. Constructing a circle from four
    ///   cubic quarters ends at `sin(2π)`, which is -2.4e-16 rather than
    ///   zero, so the path misses its own start by a fraction of a nanometre.
    ///   Appending an edge there would be worse than useless: it creates a
    ///   near-zero-length segment, which is exactly the degenerate edge the
    ///   offset kernel refuses to find a normal for.
    ///
    /// The threshold is relative to the coordinates involved, so it means
    /// the same thing on a 10mm buttonhole and a 2000mm bolt of cloth.
    pub fn closed(start: Point2, mut segments: Vec<EdgeSegment>) -> Result<Self, GeometryError> {
        let Some(last) = segments.last().copied() else {
            return Self::new(start, segments);
        };

        let end = last.end();
        let gap = end.distance(&start);
        if gap == 0.0 {
            return Self::new(start, segments);
        }

        let scale = start
            .x
            .abs()
            .max(start.y.abs())
            .max(end.x.abs())
            .max(end.y.abs())
            .max(1.0);

        if gap <= CLOSURE_SNAP_RELATIVE * scale {
            let snapped = match last {
                EdgeSegment::Line { .. } => EdgeSegment::Line { to: start },
                EdgeSegment::Cubic { c1, c2, .. } => EdgeSegment::Cubic { c1, c2, to: start },
            };
            *segments.last_mut().expect("checked non-empty") = snapped;
        } else {
            segments.push(EdgeSegment::Line { to: start });
        }

        Self::new(start, segments)
    }

    pub fn start(&self) -> Point2 {
        self.start
    }

    /// The edges that walk this path back to its start.
    ///
    /// Named `edges`, not `segments`, because it no longer returns segments.
    /// A name that does not describe what it returns is worse than the churn
    /// of renaming it.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Approximates the path as a polygon whose deviation from the true
    /// curve is at most `tolerance_mm`.
    ///
    /// Adaptive: straight stretches cost one segment, tight corners get
    /// subdivided until they earn their vertices. Flatness is measured as
    /// the distance of the control points from the chord, which bounds the
    /// curve's own deviation because a Bézier lies inside its control hull.
    pub fn flatten(&self, tolerance_mm: f64) -> Result<PatternBoundary, GeometryError> {
        if !tolerance_mm.is_finite() || tolerance_mm <= 0.0 {
            return Err(GeometryError::ToleranceNotPositive { tolerance_mm });
        }

        let mut points = vec![self.start];
        let mut cursor = self.start;

        for edge in &self.edges {
            match edge.geometry() {
                EdgeSegment::Line { to } => points.push(to),
                EdgeSegment::Cubic { c1, c2, to } => {
                    flatten_cubic(cursor, c1, c2, to, tolerance_mm, 0, &mut points);
                    points.push(to);
                }
            }
            cursor = edge.end();
        }

        // The path closes, so the final point repeats the first.
        // `PatternBoundary::new` drops it, along with any duplicate a
        // zero-length segment produced.
        PatternBoundary::new(points)
    }

    /// [`SeamPath::flatten`], tightened so the tolerance still holds *after*
    /// the result is offset by `offset_mm`.
    ///
    /// Flattening error is a sagitta, proportional to the local radius, and
    /// offsetting changes that radius: a region of curvature κ offset by a
    /// signed distance `d` along the outward normal has its error scaled by
    /// `1 + d·κ`. The sign of `d` decides *which* regions grow rather than
    /// whether any do — a convex region amplifies when offset outward and
    /// shrinks when inset, a concave region does the opposite — so the
    /// worst case is `1 + |d|·|κ_max|` in both directions, and tightening
    /// by its reciprocal is conservative either way.
    ///
    /// Treat that formula as an implementation detail. The specification is
    /// the circle oracle in this module's tests, which asserts the result
    /// against a closed-form answer across convex, concave, outset and
    /// inset.
    ///
    /// The tight factor is actually `max(1, |d|·κ)` rather than `1 + |d|·κ`
    /// — worked through in `docs/adr/ADR-003-curve-representation.md` — so
    /// this over-tightens by up to 2x near `|d|·κ ≈ 1`. Kept deliberately:
    /// the derivation assumes locally uniform chords, adaptive subdivision
    /// of a varying-curvature cubic does not produce them, and the slack
    /// costs about 1% of a 120Hz frame.
    pub fn flatten_for_offset(
        &self,
        tolerance_mm: f64,
        offset_mm: f64,
    ) -> Result<PatternBoundary, GeometryError> {
        if !tolerance_mm.is_finite() || tolerance_mm <= 0.0 {
            return Err(GeometryError::ToleranceNotPositive { tolerance_mm });
        }
        if !offset_mm.is_finite() {
            return Err(GeometryError::NonFiniteDistance {
                distance_mm: offset_mm,
            });
        }

        let amplification =
            (1.0 + offset_mm.abs() * self.max_curvature()).clamp(1.0, MAX_TOLERANCE_TIGHTENING);
        self.flatten(tolerance_mm / amplification)
    }

    /// The largest curvature found along the path, in reciprocal
    /// millimetres. Straight segments contribute nothing.
    ///
    /// Estimated by sampling rather than solved in closed form. A cubic's
    /// curvature extremum is a root of a high-degree polynomial, and cubics
    /// admit cusps where curvature is unbounded, so there is no finite
    /// analytic bound to compute for arbitrary input. Samples where the
    /// first derivative vanishes are skipped: curvature is undefined at a
    /// cusp, and the clamp in [`SeamPath::flatten_for_offset`] is what
    /// keeps that from mattering.
    pub fn max_curvature(&self) -> f64 {
        let mut cursor = self.start;
        let mut max = 0.0f64;

        for edge in &self.edges {
            if let EdgeSegment::Cubic { c1, c2, to } = edge.geometry() {
                for i in 0..=CURVATURE_SAMPLES {
                    let t = i as f64 / CURVATURE_SAMPLES as f64;
                    if let Some(k) = cubic_curvature(cursor, c1, c2, to, t) {
                        max = max.max(k);
                    }
                }
            }
            cursor = edge.end();
        }

        max
    }
}

fn midpoint(a: Point2, b: Point2) -> Point2 {
    Point2::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}

/// Distance from `p` to the infinite line through `a` and `b`, or to `a`
/// itself when the chord has collapsed to a point.
fn distance_to_chord(p: Point2, a: Point2, b: Point2) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let length = dx.hypot(dy);
    if length == 0.0 {
        return p.distance(&a);
    }
    ((p.x - a.x) * dy - (p.y - a.y) * dx).abs() / length
}

/// Emits interior points for the cubic `p0..p3`. Neither endpoint is
/// emitted; the caller owns those, which keeps segments from duplicating
/// their shared vertices.
fn flatten_cubic(
    p0: Point2,
    p1: Point2,
    p2: Point2,
    p3: Point2,
    tolerance_mm: f64,
    depth: u8,
    out: &mut Vec<Point2>,
) {
    let flatness = distance_to_chord(p1, p0, p3).max(distance_to_chord(p2, p0, p3));

    if flatness <= tolerance_mm || depth >= MAX_SUBDIVISION_DEPTH {
        return;
    }

    // de Casteljau at t = 0.5.
    let p01 = midpoint(p0, p1);
    let p12 = midpoint(p1, p2);
    let p23 = midpoint(p2, p3);
    let p012 = midpoint(p01, p12);
    let p123 = midpoint(p12, p23);
    let mid = midpoint(p012, p123);

    flatten_cubic(p0, p01, p012, mid, tolerance_mm, depth + 1, out);
    out.push(mid);
    flatten_cubic(mid, p123, p23, p3, tolerance_mm, depth + 1, out);
}

/// Curvature of the cubic at `t`, or `None` at a point where the first
/// derivative vanishes and curvature is undefined.
fn cubic_curvature(p0: Point2, p1: Point2, p2: Point2, p3: Point2, t: f64) -> Option<f64> {
    let u = 1.0 - t;

    let dx =
        3.0 * u * u * (p1.x - p0.x) + 6.0 * u * t * (p2.x - p1.x) + 3.0 * t * t * (p3.x - p2.x);
    let dy =
        3.0 * u * u * (p1.y - p0.y) + 6.0 * u * t * (p2.y - p1.y) + 3.0 * t * t * (p3.y - p2.y);

    let ddx = 6.0 * u * (p2.x - 2.0 * p1.x + p0.x) + 6.0 * t * (p3.x - 2.0 * p2.x + p1.x);
    let ddy = 6.0 * u * (p2.y - 2.0 * p1.y + p0.y) + 6.0 * t * (p3.y - 2.0 * p2.y + p1.y);

    let speed = dx.hypot(dy);
    let denominator = speed * speed * speed;
    if !denominator.is_finite() || denominator <= 0.0 {
        return None;
    }

    let curvature = (dx * ddy - dy * ddx).abs() / denominator;
    curvature.is_finite().then_some(curvature)
}
