//! Core 2D geometry primitives for pattern pieces.
//!
//! This crate is deliberately platform- and UI-agnostic: it is the
//! mathematical substrate the rest of the engine (and eventually every
//! front end) builds on.

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

    pub fn distance(&self, other: &Point2) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// The closed outline of a pattern piece, as a straight-edged polygon.
///
/// Curved seam edges (necklines, armholes, hems) are a planned extension —
/// this v1 boundary intentionally only supports straight edges so the offset
/// math below stays exact rather than approximated.
#[derive(Debug, Clone, PartialEq)]
pub struct PatternBoundary {
    pub points: Vec<Point2>,
}

impl PatternBoundary {
    pub fn new(points: Vec<Point2>) -> Self {
        Self { points }
    }

    pub fn perimeter(&self) -> f64 {
        if self.points.len() < 2 {
            return 0.0;
        }
        let n = self.points.len();
        (0..n)
            .map(|i| self.points[i].distance(&self.points[(i + 1) % n]))
            .sum()
    }

    /// The boundary is wound counter-clockwise (in standard math orientation,
    /// y-up) when this returns `true`.
    pub fn is_counter_clockwise(&self) -> bool {
        self.signed_area() > 0.0
    }

    fn signed_area(&self) -> f64 {
        let n = self.points.len();
        if n < 3 {
            return 0.0;
        }
        let sum: f64 = (0..n)
            .map(|i| {
                let a = self.points[i];
                let b = self.points[(i + 1) % n];
                a.x * b.y - b.x * a.y
            })
            .sum();
        sum / 2.0
    }

    /// Offsets every edge outward along its normal by `distance` millimeters,
    /// then re-intersects consecutive edges to find the new vertices. This is
    /// the seam-allowance construction: a positive distance expands the
    /// boundary regardless of its winding direction.
    pub fn offset(&self, distance: f64) -> PatternBoundary {
        let n = self.points.len();
        if n < 3 || distance == 0.0 {
            return self.clone();
        }

        let orientation = if self.is_counter_clockwise() { 1.0 } else { -1.0 };

        let offset_edges: Vec<(Point2, Point2)> = (0..n)
            .map(|i| {
                let a = self.points[i];
                let b = self.points[(i + 1) % n];
                let edge = Point2::new(b.x - a.x, b.y - a.y);
                let len = (edge.x.powi(2) + edge.y.powi(2)).sqrt();
                // Rotating the edge direction by -90 degrees gives the
                // outward normal for a counter-clockwise polygon; the
                // `orientation` factor flips it outward for clockwise ones.
                let normal = Point2::new(edge.y / len, -edge.x / len);
                let push = orientation * distance;
                (
                    Point2::new(a.x + normal.x * push, a.y + normal.y * push),
                    Point2::new(b.x + normal.x * push, b.y + normal.y * push),
                )
            })
            .collect();

        let new_points: Vec<Point2> = (0..n)
            .map(|i| {
                let prev_edge = offset_edges[(i + n - 1) % n];
                let curr_edge = offset_edges[i];
                line_intersection(prev_edge, curr_edge).unwrap_or(curr_edge.0)
            })
            .collect();

        PatternBoundary::new(new_points)
    }
}

/// Intersection of two infinite lines, each defined by a pair of points.
/// Returns `None` for parallel (or nearly parallel) lines.
fn line_intersection(l1: (Point2, Point2), l2: (Point2, Point2)) -> Option<Point2> {
    let (p1, p2) = l1;
    let (p3, p4) = l2;
    let d1 = Point2::new(p2.x - p1.x, p2.y - p1.y);
    let d2 = Point2::new(p4.x - p3.x, p4.y - p3.y);
    let denom = d1.x * d2.y - d1.y * d2.x;
    if denom.abs() < 1e-9 {
        return None;
    }
    let t = ((p3.x - p1.x) * d2.y - (p3.y - p1.y) * d2.x) / denom;
    Some(Point2::new(p1.x + t * d1.x, p1.y + t * d1.y))
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
    }

    #[test]
    fn perimeter_of_unit_square() {
        assert_eq!(square(10.0).perimeter(), 40.0);
    }

    #[test]
    fn square_wound_counter_clockwise() {
        assert!(square(10.0).is_counter_clockwise());
    }

    #[test]
    fn reversed_square_is_clockwise() {
        let mut pts = square(10.0).points;
        pts.reverse();
        assert!(!PatternBoundary::new(pts).is_counter_clockwise());
    }

    #[test]
    fn offset_expands_square_symmetrically() {
        let expanded = square(10.0).offset(1.0);
        let expected = [
            Point2::new(-1.0, -1.0),
            Point2::new(11.0, -1.0),
            Point2::new(11.0, 11.0),
            Point2::new(-1.0, 11.0),
        ];
        for (actual, expected) in expanded.points.iter().zip(expected.iter()) {
            assert!((actual.x - expected.x).abs() < 1e-6);
            assert!((actual.y - expected.y).abs() < 1e-6);
        }
    }

    #[test]
    fn offset_is_orientation_independent() {
        let mut reversed_pts = square(10.0).points;
        reversed_pts.reverse();
        let reversed = PatternBoundary::new(reversed_pts);
        let expanded = reversed.offset(1.0);
        // Expanding should always grow the perimeter, regardless of winding.
        assert!(expanded.perimeter() > reversed.perimeter());
    }

    #[test]
    fn distance_between_points() {
        let a = Point2::new(0.0, 0.0);
        let b = Point2::new(3.0, 4.0);
        assert_eq!(a.distance(&b), 5.0);
    }
}
