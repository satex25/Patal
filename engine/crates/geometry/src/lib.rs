//! Core 2D geometry primitives for pattern pieces.
//!
//! This crate is deliberately platform- and UI-agnostic: it is the
//! mathematical substrate the rest of the engine (and eventually every
//! front end) builds on.
//!
//! Every operation here is either correct or loud. A pattern piece that is
//! silently wrong is worse than one that refuses to compute: the first gets
//! cut out of cloth, the second gets fixed. So the fallible operations return
//! [`GeometryError`] rather than a plausible-looking number, and
//! [`PatternBoundary`] owns its points privately so its invariants cannot be
//! broken after construction.

#![forbid(unsafe_code)]

use std::fmt;

/// Everything that can go wrong in this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryError {
    /// A coordinate was NaN or infinite. Non-finite input poisons every
    /// downstream number, so it is rejected at the boundary rather than
    /// propagated.
    NonFiniteCoordinate { index: usize },
    /// A closed boundary needs at least three distinct points. Fewer cannot
    /// enclose an area, so there is no inside to offset away from.
    TooFewPoints { count: usize },
    /// An edge had zero (or unrepresentably small) length, so it has no
    /// direction and therefore no normal to offset along.
    DegenerateEdge { index: usize },
    /// The offset distance was NaN or infinite.
    NonFiniteDistance { distance_mm: f64 },
    /// The boundary encloses no measurable area, so "outward" is undefined
    /// and a seam allowance cannot be pushed in a known direction.
    IndeterminateWinding,
    /// The offset consumed the piece: an inset larger than the local feature
    /// size collapses the boundary and then re-expands it inside out.
    OffsetCollapsed { distance_mm: f64 },
    /// The offset outline crosses itself — the classic dart/notch failure,
    /// where a concave feature narrower than twice the offset turns inside
    /// out instead of vanishing.
    OffsetSelfIntersects { distance_mm: f64 },
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteCoordinate { index } => {
                write!(f, "point {index} has a non-finite coordinate")
            }
            Self::TooFewPoints { count } => write!(
                f,
                "a closed boundary needs at least 3 distinct points, got {count}"
            ),
            Self::DegenerateEdge { index } => {
                write!(f, "edge {index} has zero length and no direction")
            }
            Self::NonFiniteDistance { distance_mm } => {
                write!(f, "offset distance {distance_mm} is not finite")
            }
            Self::IndeterminateWinding => {
                write!(f, "boundary encloses no area, so its winding is undetermined")
            }
            Self::OffsetCollapsed { distance_mm } => write!(
                f,
                "offsetting by {distance_mm}mm collapsed the boundary through itself"
            ),
            Self::OffsetSelfIntersects { distance_mm } => write!(
                f,
                "offsetting by {distance_mm}mm produced a self-intersecting outline"
            ),
        }
    }
}

impl std::error::Error for GeometryError {}

/// Which way a boundary is wound, or that the question has no answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Winding {
    CounterClockwise,
    Clockwise,
    /// The boundary encloses no measurable area — a fully collinear point
    /// list, or a figure-eight whose lobes cancel exactly.
    Degenerate,
}

/// A point in pattern space, in millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Euclidean distance. Uses `hypot` rather than `(dx² + dy²).sqrt()` so
    /// that large coordinates do not overflow to infinity and small ones do
    /// not underflow to zero.
    pub fn distance(&self, other: &Point2) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

/// The closed outline of a pattern piece, as a straight-edged polygon.
///
/// Curved seam edges (necklines, armholes, hems) are a planned extension —
/// this v1 boundary only supports straight edges, so a curve must be
/// approximated by a polyline and its length will read slightly short.
///
/// # Invariants
///
/// Construction is the only way in, so every `PatternBoundary` in existence
/// has at least 3 points, all finite, with no two consecutive points equal
/// and no repeated closing point. `points` is private to keep that true for
/// the value's whole life rather than only at its birth.
#[derive(Debug, Clone, PartialEq)]
pub struct PatternBoundary {
    points: Vec<Point2>,
}

/// Corner joins longer than this multiple of the offset distance are cut off
/// square (bevelled) instead of mitred. Without a cap the mitre length grows
/// as `1 / sin(θ/2)`, so an ordinary 6° collar point or dart apex would throw
/// the offset vertex hundreds of millimetres clear of the piece.
const DEFAULT_MITER_LIMIT: f64 = 4.0;

/// Two join points closer than this fraction of the offset distance are the
/// same corner reported twice; keep one.
const JOIN_MERGE_FRACTION: f64 = 1e-9;

/// Relative tolerance for calling a signed area zero. Area scales with the
/// square of length, so this is applied against `perimeter²` — an absolute
/// epsilon would call a metre-scale piece degenerate and a millimetre-scale
/// one well-formed.
const AREA_EPS: f64 = 1e-12;

/// Minimum `|sin θ|` between two edges before they count as parallel.
const SIN_EPS: f64 = 1e-9;

impl PatternBoundary {
    /// Validates and normalises a point list into a closed boundary.
    ///
    /// Consecutive duplicate points are dropped (they carry no direction, and
    /// dividing by their zero-length edge is what used to poison an entire
    /// outline with NaN), as is a closing point that repeats the first.
    pub fn new(points: Vec<Point2>) -> Result<Self, GeometryError> {
        for (index, point) in points.iter().enumerate() {
            if !point.is_finite() {
                return Err(GeometryError::NonFiniteCoordinate { index });
            }
        }

        let points = dedup_closed(points);
        if points.len() < 3 {
            return Err(GeometryError::TooFewPoints {
                count: points.len(),
            });
        }

        Ok(Self { points })
    }

    pub fn points(&self) -> &[Point2] {
        &self.points
    }

    pub fn into_points(self) -> Vec<Point2> {
        self.points
    }

    pub fn perimeter(&self) -> f64 {
        let n = self.points.len();
        (0..n)
            .map(|i| self.points[i].distance(&self.points[(i + 1) % n]))
            .sum()
    }

    pub fn signed_area(&self) -> f64 {
        let n = self.points.len();
        let sum: f64 = (0..n)
            .map(|i| {
                let a = self.points[i];
                let b = self.points[(i + 1) % n];
                a.x * b.y - b.x * a.y
            })
            .sum();
        sum / 2.0
    }

    /// Which way the boundary is wound, in standard math orientation (y-up).
    ///
    /// Returns [`Winding::Degenerate`] rather than guessing when the enclosed
    /// area is indistinguishable from zero — a bare `> 0.0` on the shoelace
    /// sum silently reports "clockwise" for those, which flips the seam
    /// allowance inward on the whole piece.
    pub fn winding(&self) -> Winding {
        let area = self.signed_area();
        let scale = self.perimeter();
        let tolerance = scale * scale * AREA_EPS;
        if area > tolerance {
            Winding::CounterClockwise
        } else if area < -tolerance {
            Winding::Clockwise
        } else {
            Winding::Degenerate
        }
    }

    /// Whether any two non-adjacent edges cross.
    ///
    /// `O(n²)`, which is the right trade at pattern-piece sizes (tens to low
    /// hundreds of vertices) and keeps the check exact.
    pub fn self_intersects(&self) -> bool {
        let n = self.points.len();
        for i in 0..n {
            let a1 = self.points[i];
            let a2 = self.points[(i + 1) % n];
            for j in (i + 1)..n {
                // Skip edges sharing a vertex: they meet by construction.
                if (i + 1) % n == j || (j + 1) % n == i {
                    continue;
                }
                let b1 = self.points[j];
                let b2 = self.points[(j + 1) % n];
                if segments_properly_intersect(a1, a2, b1, b2) {
                    return true;
                }
            }
        }
        false
    }

    /// Offsets every edge along its outward normal by `distance_mm`, then
    /// re-intersects consecutive edges to find the new vertices. This is the
    /// seam-allowance construction: a positive distance expands the boundary
    /// regardless of winding, a negative one insets it.
    ///
    /// The result is checked before it is returned — an offset that collapses
    /// the piece through itself or produces a crossing outline is an error,
    /// not a returned polygon.
    pub fn offset(&self, distance_mm: f64) -> Result<PatternBoundary, GeometryError> {
        self.offset_with_miter_limit(distance_mm, DEFAULT_MITER_LIMIT)
    }

    /// [`offset`](Self::offset) with an explicit mitre limit, for callers that
    /// want sharper points than the default allows.
    pub fn offset_with_miter_limit(
        &self,
        distance_mm: f64,
        miter_limit: f64,
    ) -> Result<PatternBoundary, GeometryError> {
        if !distance_mm.is_finite() {
            return Err(GeometryError::NonFiniteDistance { distance_mm });
        }
        if distance_mm == 0.0 {
            return Ok(self.clone());
        }

        let winding = self.winding();
        let orientation = match winding {
            Winding::CounterClockwise => 1.0,
            Winding::Clockwise => -1.0,
            Winding::Degenerate => return Err(GeometryError::IndeterminateWinding),
        };

        let n = self.points.len();
        let push = orientation * distance_mm;
        let mut offset_edges = Vec::with_capacity(n);
        for i in 0..n {
            let a = self.points[i];
            let b = self.points[(i + 1) % n];
            let edge = Point2::new(b.x - a.x, b.y - a.y);
            let len = edge.x.hypot(edge.y);
            // Rotating the edge direction by -90 degrees gives the outward
            // normal for a counter-clockwise polygon; `orientation` flips it
            // outward for clockwise ones.
            let normal = Point2::new(edge.y / len, -edge.x / len);
            if !normal.is_finite() {
                return Err(GeometryError::DegenerateEdge { index: i });
            }
            offset_edges.push((
                Point2::new(a.x + normal.x * push, a.y + normal.y * push),
                Point2::new(b.x + normal.x * push, b.y + normal.y * push),
            ));
        }

        let miter_cap = miter_limit.abs() * distance_mm.abs();
        // Where each source vertex's join landed in `new_points`. A bevel
        // emits two points, so this is a range rather than an index.
        let mut joins: Vec<(usize, usize)> = Vec::with_capacity(n);
        let mut new_points: Vec<Point2> = Vec::with_capacity(n);
        for i in 0..n {
            let prev = offset_edges[(i + n - 1) % n];
            let curr = offset_edges[i];
            let vertex = self.points[i];
            let first = new_points.len();

            match line_intersection(prev, curr) {
                // A mitre we can afford: the two offset edges meet at a point
                // close enough to the original corner to be a real corner.
                Some(joined) if joined.distance(&vertex) <= miter_cap => {
                    new_points.push(joined);
                }
                // Either the mitre ran away (an acute point) or the edges are
                // parallel and never meet. Both are answered by a bevel: walk
                // the end of the previous offset edge to the start of this
                // one. Never fabricate a vertex that lies on neither line.
                _ => {
                    if prev.1.distance(&curr.0) <= JOIN_MERGE_FRACTION * distance_mm.abs() {
                        new_points.push(curr.0);
                    } else {
                        new_points.push(prev.1);
                        new_points.push(curr.0);
                    }
                }
            }

            joins.push((first, new_points.len() - 1));
        }

        // The validity test that matters: an offset edge must still run the
        // same way as the edge it came from. When an inset eats more than the
        // piece has to give, opposite edges swap places and every edge
        // reverses — and because that is a 180-degree rotation, the winding
        // and the area sign both survive it. Checking the winding would pass
        // a 20mm square "inset" by 15mm into a plausible 10mm square.
        for i in 0..n {
            let a = self.points[i];
            let b = self.points[(i + 1) % n];
            let source = Point2::new(b.x - a.x, b.y - a.y);

            let start = new_points[joins[i].1];
            let end = new_points[joins[(i + 1) % n].0];
            let offset_direction = Point2::new(end.x - start.x, end.y - start.y);

            if source.x * offset_direction.x + source.y * offset_direction.y <= 0.0 {
                return Err(GeometryError::OffsetCollapsed { distance_mm });
            }
        }

        let offset = PatternBoundary::new(new_points)
            .map_err(|_| GeometryError::OffsetCollapsed { distance_mm })?;

        if offset.self_intersects() {
            return Err(GeometryError::OffsetSelfIntersects { distance_mm });
        }

        Ok(offset)
    }
}

/// Drops consecutive duplicate points and any closing point that repeats the
/// first, leaving a clean cyclic sequence.
fn dedup_closed(points: Vec<Point2>) -> Vec<Point2> {
    let mut out: Vec<Point2> = Vec::with_capacity(points.len());
    for point in points {
        if out.last() != Some(&point) {
            out.push(point);
        }
    }
    while out.len() > 1 && out[0] == out[out.len() - 1] {
        out.pop();
    }
    out
}

/// Intersection of two infinite lines, each defined by a pair of points.
/// Returns `None` when they are parallel to within [`SIN_EPS`].
fn line_intersection(l1: (Point2, Point2), l2: (Point2, Point2)) -> Option<Point2> {
    let (p1, p2) = l1;
    let (p3, p4) = l2;
    let d1 = Point2::new(p2.x - p1.x, p2.y - p1.y);
    let d2 = Point2::new(p4.x - p3.x, p4.y - p3.y);
    let denom = d1.x * d2.y - d1.y * d2.x;

    // `denom` is |d1||d2|sin(θ), so testing it against a fixed epsilon would
    // make the parallelism decision depend on edge length rather than on the
    // angle. Normalise first: the question is about direction only.
    let scale = d1.x.hypot(d1.y) * d2.x.hypot(d2.y);
    if scale == 0.0 || (denom / scale).abs() < SIN_EPS {
        return None;
    }

    let t = ((p3.x - p1.x) * d2.y - (p3.y - p1.y) * d2.x) / denom;
    let intersection = Point2::new(p1.x + t * d1.x, p1.y + t * d1.y);
    intersection.is_finite().then_some(intersection)
}

/// Cross product of `oa` and `ob` — positive when `o -> a -> b` turns left.
fn cross(o: Point2, a: Point2, b: Point2) -> f64 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

/// Whether two segments cross at an interior point of both. Touching
/// endpoints and collinear overlap do not count — adjacent polygon edges
/// always share a vertex, and that is not a self-intersection.
fn segments_properly_intersect(p1: Point2, p2: Point2, p3: Point2, p4: Point2) -> bool {
    let d1 = cross(p3, p4, p1);
    let d2 = cross(p3, p4, p2);
    let d3 = cross(p1, p2, p3);
    let d4 = cross(p1, p2, p4);

    ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(side: f64) -> PatternBoundary {
        PatternBoundary::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(side, 0.0),
            Point2::new(side, side),
            Point2::new(0.0, side),
        ])
        .expect("square is a valid boundary")
    }

    #[test]
    fn perimeter_of_unit_square() {
        assert_eq!(square(10.0).perimeter(), 40.0);
    }

    #[test]
    fn square_wound_counter_clockwise() {
        assert_eq!(square(10.0).winding(), Winding::CounterClockwise);
    }

    #[test]
    fn reversed_square_is_clockwise() {
        let mut pts = square(10.0).into_points();
        pts.reverse();
        let reversed = PatternBoundary::new(pts).expect("still a valid boundary");
        assert_eq!(reversed.winding(), Winding::Clockwise);
    }

    #[test]
    fn offset_expands_square_symmetrically() {
        let expanded = square(10.0).offset(1.0).expect("square offsets cleanly");
        let expected = [
            Point2::new(-1.0, -1.0),
            Point2::new(11.0, -1.0),
            Point2::new(11.0, 11.0),
            Point2::new(-1.0, 11.0),
        ];
        assert_eq!(expanded.points().len(), expected.len());
        for (actual, expected) in expanded.points().iter().zip(expected.iter()) {
            assert!((actual.x - expected.x).abs() < 1e-6);
            assert!((actual.y - expected.y).abs() < 1e-6);
        }
    }

    #[test]
    fn offset_is_orientation_independent() {
        let mut reversed_pts = square(10.0).into_points();
        reversed_pts.reverse();
        let reversed = PatternBoundary::new(reversed_pts).expect("valid");
        let expanded = reversed.offset(1.0).expect("offsets cleanly");
        // Expanding should always grow the perimeter, regardless of winding.
        assert!(expanded.perimeter() > reversed.perimeter());
    }

    #[test]
    fn distance_between_points() {
        let a = Point2::new(0.0, 0.0);
        let b = Point2::new(3.0, 4.0);
        assert_eq!(a.distance(&b), 5.0);
    }

    #[test]
    fn distance_survives_extreme_magnitudes() {
        // `(dx² + dy²).sqrt()` overflows here; hypot does not.
        let far = Point2::new(0.0, 0.0).distance(&Point2::new(1e200, 0.0));
        assert_eq!(far, 1e200);
        // ...and underflows here.
        let near = Point2::new(0.0, 0.0).distance(&Point2::new(1e-170, 1e-170));
        assert!(near > 0.0);
    }

    #[test]
    fn non_finite_coordinates_are_rejected() {
        let err = PatternBoundary::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(f64::NAN, 0.0),
            Point2::new(10.0, 10.0),
        ])
        .unwrap_err();
        assert_eq!(err, GeometryError::NonFiniteCoordinate { index: 1 });
    }

    #[test]
    fn duplicate_consecutive_points_are_dropped_not_divided_by() {
        // This used to produce a zero-length edge, a NaN normal, and an
        // entire boundary of NaN with no error of any kind.
        let boundary = PatternBoundary::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(100.0, 0.0),
            Point2::new(100.0, 0.0),
            Point2::new(100.0, 200.0),
            Point2::new(0.0, 200.0),
        ])
        .expect("the duplicate is removed, leaving a valid rectangle");
        assert_eq!(boundary.points().len(), 4);

        let cut = boundary.offset(10.0).expect("offsets cleanly");
        assert!(cut.points().iter().all(Point2::is_finite));
        assert!(cut.perimeter().is_finite());
    }

    #[test]
    fn closing_point_that_repeats_the_first_is_dropped() {
        let boundary = PatternBoundary::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(10.0, 0.0),
            Point2::new(10.0, 10.0),
            Point2::new(0.0, 0.0),
        ])
        .expect("valid triangle");
        assert_eq!(boundary.points().len(), 3);
    }

    #[test]
    fn fewer_than_three_points_is_an_error() {
        let err = PatternBoundary::new(vec![Point2::new(0.0, 0.0), Point2::new(50.0, 0.0)])
            .unwrap_err();
        assert_eq!(err, GeometryError::TooFewPoints { count: 2 });
    }

    #[test]
    fn non_finite_offset_distance_is_an_error() {
        let err = square(10.0).offset(f64::NAN).unwrap_err();
        assert!(matches!(err, GeometryError::NonFiniteDistance { .. }));
    }

    #[test]
    fn zero_area_boundary_has_no_offset_direction() {
        // A bow-tie: the lobes cancel, so the shoelace sum is exactly zero and
        // "outward" has no meaning. This used to be silently called clockwise.
        let bowtie = PatternBoundary::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(100.0, 100.0),
            Point2::new(100.0, 0.0),
            Point2::new(0.0, 100.0),
        ])
        .expect("four distinct finite points");
        assert_eq!(bowtie.winding(), Winding::Degenerate);
        assert_eq!(
            bowtie.offset(10.0).unwrap_err(),
            GeometryError::IndeterminateWinding
        );
    }

    #[test]
    fn acute_point_is_bevelled_not_flung_away() {
        // A 6-degree dart apex. Un-capped, the mitre puts the offset vertex
        // ~190mm from the original corner on a 200mm-tall piece.
        let spike = PatternBoundary::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(100.0, 0.0),
            Point2::new(100.0, 200.0),
            Point2::new(94.75, 100.0),
            Point2::new(0.0, 200.0),
        ])
        .expect("valid");

        let distance = 10.0;
        let offset = spike.offset(distance).expect("offsets cleanly");
        let cap = DEFAULT_MITER_LIMIT * distance;
        for point in offset.points() {
            let nearest = spike
                .points()
                .iter()
                .map(|original| original.distance(point))
                .fold(f64::INFINITY, f64::min);
            assert!(
                nearest <= cap + 1e-9,
                "join at {point:?} is {nearest}mm from the outline, past the {cap}mm cap"
            );
        }
    }

    #[test]
    fn over_inset_reports_collapse_instead_of_inverting() {
        // A 20mm square inset by 15mm used to return a valid-looking 10mm
        // square with inverted winding and a positive area.
        let err = square(20.0).offset(-15.0).unwrap_err();
        assert!(matches!(err, GeometryError::OffsetCollapsed { .. }));
    }

    #[test]
    fn small_inset_still_works() {
        let inset = square(20.0).offset(-2.0).expect("2mm inset fits easily");
        assert!(inset.perimeter() < square(20.0).perimeter());
        assert_eq!(inset.winding(), Winding::CounterClockwise);
    }

    #[test]
    fn concave_slot_narrower_than_the_offset_is_reported() {
        // A 14mm-wide, 30mm-deep slot with a 10mm allowance: the two sides of
        // the slot cross. This used to be returned as a valid cut line.
        let slotted = PatternBoundary::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(100.0, 0.0),
            Point2::new(100.0, 60.0),
            Point2::new(57.0, 60.0),
            Point2::new(57.0, 30.0),
            Point2::new(43.0, 30.0),
            Point2::new(43.0, 60.0),
            Point2::new(0.0, 60.0),
        ])
        .expect("valid");

        let err = slotted.offset(10.0).unwrap_err();
        assert!(matches!(
            err,
            GeometryError::OffsetSelfIntersects { .. } | GeometryError::OffsetCollapsed { .. }
        ));
    }

    #[test]
    fn line_intersection_is_scale_independent() {
        // A well-conditioned right angle at small scale: the old absolute
        // 1e-9 test on an unnormalised cross product called this parallel.
        let side = 3e-5;
        let tiny = PatternBoundary::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(side, 0.0),
            Point2::new(side, side),
            Point2::new(0.0, side),
        ])
        .expect("valid");

        let expanded = tiny.offset(side * 0.1).expect("offsets cleanly");
        let corner = expanded.points()[0];
        assert!((corner.x - -side * 0.1).abs() < side * 1e-6);
        assert!((corner.y - -side * 0.1).abs() < side * 1e-6);
    }

    #[test]
    fn self_intersection_detects_a_crossing_outline() {
        let bowtie = PatternBoundary::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(100.0, 100.0),
            Point2::new(100.0, 0.0),
            Point2::new(0.0, 100.0),
        ])
        .expect("valid");
        assert!(bowtie.self_intersects());
        assert!(!square(10.0).self_intersects());
    }
}
