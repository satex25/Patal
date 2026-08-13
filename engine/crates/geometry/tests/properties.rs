//! Property-based invariants for the offset kernel.
//!
//! The 30 unit tests in `lib.rs` prove the cases someone thought of. These
//! prove the cases nobody thought of, on inputs nobody would write down.
//!
//! An honest note about what this is for. The hardening plan introduced this
//! suite as "a safety net landing before anything near the kernel changes" —
//! but the plan also states in three places that the kernel is not modified,
//! so there was nothing for the net to catch. Its real value is
//! latent-defect discovery in code that already shipped: finding, in
//! already-released behaviour, the input that panics or returns a boundary
//! that quietly crosses itself. That is worth having on its own terms, and
//! it does not need a false justification.
//!
//! Deliberately an integration test rather than a `#[cfg(test)]` module: it
//! can only reach the public API, which is exactly the surface whose
//! contract these properties describe.

use patal_geometry::{GeometryError, PatternBoundary, Point2, Winding};
use proptest::prelude::*;

/// Coordinates stay inside a plausible garment-scale envelope. Extreme
/// magnitudes get their own generator below rather than diluting this one:
/// mixing 1e200 into every case means most runs test float saturation
/// instead of geometry.
const COORD: std::ops::Range<f64> = -1000.0..1000.0;

/// Convex hull by Andrew's monotone chain, returned counter-clockwise.
///
/// Random points make a *self-intersecting* polygon far more often than a
/// simple one, so generating valid convex boundaries means constructing
/// them rather than filtering for them.
fn convex_hull(mut points: Vec<Point2>) -> Vec<Point2> {
    points.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap()
            .then(a.y.partial_cmp(&b.y).unwrap())
    });
    points.dedup_by(|a, b| a.x == b.x && a.y == b.y);
    if points.len() < 3 {
        return points;
    }

    let cross =
        |o: Point2, a: Point2, b: Point2| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);

    let mut hull: Vec<Point2> = Vec::with_capacity(points.len() * 2);
    for &p in points.iter() {
        while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0 {
            hull.pop();
        }
        hull.push(p);
    }
    let lower = hull.len() + 1;
    for &p in points.iter().rev() {
        while hull.len() >= lower && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0 {
            hull.pop();
        }
        hull.push(p);
    }
    hull.pop();
    hull
}

prop_compose! {
    /// A guaranteed-valid convex boundary that encloses real area.
    ///
    /// The winding filter matters: a hull of three nearly collinear points
    /// is a valid `PatternBoundary` whose winding is `Degenerate`, and
    /// `offset` correctly refuses those. Including them would make every
    /// property below have to special-case an error that is not a defect.
    fn convex_boundary()(
        raw in prop::collection::vec((COORD, COORD), 3..14)
    ) -> Option<PatternBoundary> {
        let points: Vec<Point2> = raw.into_iter().map(|(x, y)| Point2::new(x, y)).collect();
        let hull = convex_hull(points);
        PatternBoundary::new(hull)
            .ok()
            .filter(|b| b.winding() != Winding::Degenerate)
    }
}

prop_compose! {
    /// Arbitrary point soup: concave, self-crossing, duplicated, extreme,
    /// and non-finite. Most of this never becomes a boundary at all — the
    /// point is that whatever survives construction must still be handled.
    fn point_soup()(
        raw in prop::collection::vec(
            prop_oneof![
                4 => (COORD, COORD),
                1 => (-1e200..1e200, -1e200..1e200),
                1 => (-1e-170..1e-170, -1e-170..1e-170),
                1 => Just((f64::NAN, 0.0)),
                1 => Just((f64::INFINITY, f64::NEG_INFINITY)),
            ],
            0..12,
        )
    ) -> Vec<Point2> {
        raw.into_iter().map(|(x, y)| Point2::new(x, y)).collect()
    }
}

proptest! {
    // ---------------------------------------------------------------
    // Convex generator: the shapes where outward offset has a
    // guaranteed answer, so these assert success rather than tolerate
    // failure.
    // ---------------------------------------------------------------

    /// Outward offset of a convex piece cannot fail. Every join is a mitre
    /// or a bevel pushed outward, no edge can reverse, and a convex outline
    /// pushed outward cannot cross itself. If this ever goes red, the
    /// kernel is refusing work it should be doing.
    #[test]
    fn outward_offset_of_a_convex_piece_always_succeeds(
        boundary in convex_boundary(),
        distance in 0.001f64..50.0,
    ) {
        let Some(boundary) = boundary else { return Ok(()) };
        prop_assert!(
            boundary.offset(distance).is_ok(),
            "convex outward offset by {distance} failed: {:?}",
            boundary.offset(distance).unwrap_err()
        );
    }

    #[test]
    fn outward_offset_grows_perimeter_and_area(
        boundary in convex_boundary(),
        distance in 0.001f64..50.0,
    ) {
        let Some(boundary) = boundary else { return Ok(()) };
        let expanded = boundary.offset(distance).unwrap();
        prop_assert!(expanded.perimeter() > boundary.perimeter());
        prop_assert!(expanded.signed_area().abs() > boundary.signed_area().abs());
    }

    #[test]
    fn zero_offset_is_the_identity(boundary in convex_boundary()) {
        let Some(boundary) = boundary else { return Ok(()) };
        let same = boundary.offset(0.0).unwrap();
        prop_assert_eq!(same.points(), boundary.points());
    }

    #[test]
    fn winding_survives_an_outward_offset(
        boundary in convex_boundary(),
        distance in 0.001f64..50.0,
    ) {
        let Some(boundary) = boundary else { return Ok(()) };
        let expanded = boundary.offset(distance).unwrap();
        prop_assert_eq!(expanded.winding(), boundary.winding());
    }

    /// Offset out by d, back in by d, and a convex piece comes home.
    ///
    /// Restricted to convex input on purpose: on a concave shape
    /// offset-then-inset is a morphological closing, which *legitimately*
    /// fills in a notch and is not a round trip at all. Asserting it over
    /// every shape is precisely how this suite would have become
    /// flaky-by-design, then been disabled by whoever got tired of it.
    ///
    /// Within convex input there are still two regimes, and this asserts
    /// something in both rather than filtering one out. Filtering was the
    /// first attempt and it was wrong: bevels occur in roughly one case in
    /// nine, which exhausts proptest's global reject budget on any long run
    /// and fails the suite for a reason that has nothing to do with
    /// geometry.
    ///
    /// * Every join was a mitre (vertex counts match): the original
    ///   vertices come back exactly, to float tolerance.
    /// * A corner sharper than the mitre limit was bevelled: it is cut off
    ///   square on the way out and cannot be recovered on the way back.
    ///   Correct behaviour, not a defect — so the claim weakens to the one
    ///   that still holds, that a round trip never *grows* the piece.
    #[test]
    fn convex_offset_then_inset_comes_home(
        boundary in convex_boundary(),
        distance in 0.001f64..20.0,
    ) {
        let Some(boundary) = boundary else { return Ok(()) };
        let expanded = boundary.offset(distance).unwrap();
        let Ok(returned) = expanded.offset(-distance) else { return Ok(()) };

        let original_area = boundary.signed_area().abs();
        let returned_area = returned.signed_area().abs();
        prop_assert!(
            returned_area <= original_area * (1.0 + 1e-9) + 1e-9,
            "round trip grew the piece: {original_area} -> {returned_area}"
        );

        if returned.points().len() != boundary.points().len() {
            // A bevel fired. Nothing stronger than the area bound is true.
            return Ok(());
        }

        // Scale-relative: a mitre join sits at d/sin(theta/2) from its
        // corner, so float error at a sharp corner is proportional to the
        // piece, not absolute.
        let tolerance = 1e-6 * boundary.perimeter().max(1.0);
        for (before, after) in boundary.points().iter().zip(returned.points()) {
            prop_assert!(
                before.distance(after) <= tolerance,
                "{before:?} -> {after:?} moved more than {tolerance}"
            );
        }
    }

    // ---------------------------------------------------------------
    // Hostile generator: no shape guarantees at all. These assert the
    // contract that holds for *every* input — including that the
    // functions return rather than panic, which proptest checks by
    // running them.
    // ---------------------------------------------------------------

    /// The core correct-or-loud claim: whatever comes back is usable.
    /// A boundary that survives construction has finite points, and an
    /// offset that returns Ok has finite points and does not cross itself.
    /// Anything else must be an error, never a returned polygon.
    #[test]
    fn a_returned_boundary_is_always_valid(
        soup in point_soup(),
        distance in prop_oneof![-100.0f64..100.0, Just(0.0), Just(f64::NAN), Just(f64::INFINITY)],
    ) {
        let Ok(boundary) = PatternBoundary::new(soup) else { return Ok(()) };

        prop_assert!(boundary.points().iter().all(|p| p.is_finite()));
        prop_assert!(boundary.points().len() >= 3);

        let Ok(offset) = boundary.offset(distance) else { return Ok(()) };

        prop_assert!(
            offset.points().iter().all(|p| p.is_finite()),
            "offset by {distance} returned a non-finite point: {:?}",
            offset.points()
        );

        // A zero offset short-circuits to `Ok(self.clone())` before any
        // validity check runs, so it hands back a self-crossing input
        // unchanged. That is the right answer — asking for no seam
        // allowance should not be a way to make the kernel reject a shape
        // it was given — but it means the no-crossing guarantee belongs to
        // offsets that actually constructed something.
        if distance != 0.0 {
            prop_assert!(
                !offset.self_intersects(),
                "offset by {distance} returned a self-crossing cut line: {:?}",
                offset.first_self_intersection()
            );
        }
    }

    /// A non-finite distance is always an error, never a silently ignored
    /// argument or a NaN-poisoned outline.
    #[test]
    fn a_non_finite_distance_is_always_rejected(
        soup in point_soup(),
        distance in prop_oneof![Just(f64::NAN), Just(f64::INFINITY), Just(f64::NEG_INFINITY)],
    ) {
        let Ok(boundary) = PatternBoundary::new(soup) else { return Ok(()) };
        // Matched rather than compared: `GeometryError` derives PartialEq,
        // and the payload here is the distance that was rejected — so for
        // a NaN distance the error is not equal to itself.
        let result = boundary.offset(distance);
        prop_assert!(
            matches!(
                result,
                Err(GeometryError::NonFiniteDistance { distance_mm })
                    if !distance_mm.is_finite()
            ),
            "a non-finite distance must be rejected, got {result:?}"
        );
    }

    /// Serde routes through the validating constructor, so a round trip
    /// cannot lose or smuggle anything.
    #[test]
    fn serde_round_trip_is_lossless(soup in point_soup()) {
        let Ok(boundary) = PatternBoundary::new(soup) else { return Ok(()) };
        let json = serde_json::to_string(&boundary).unwrap();
        let restored: PatternBoundary = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(restored, boundary);
    }

    /// The wire format is a bare array, not a wrapped object — the shape
    /// the Swift mirror's Codable conformance is pinned against.
    #[test]
    fn the_wire_format_stays_a_bare_array(soup in point_soup()) {
        let Ok(boundary) = PatternBoundary::new(soup) else { return Ok(()) };
        let json = serde_json::to_string(&boundary).unwrap();
        prop_assert!(json.starts_with('['), "expected a bare array, got {json}");
    }
}

/// A V-shaped notch — a dart apex — is the input that actually reaches
/// `OffsetSelfIntersects`.
///
/// This was worth pinning because the obvious candidate does not. A
/// *rectangular* slot narrower than twice the allowance is caught earlier,
/// as `OffsetCollapsed`: its floor edge reverses at exactly the same
/// threshold as its walls cross, and the direction check runs first. That
/// made it look, from the unit tests alone, as though the crossing branch
/// might be unreachable.
///
/// A V-notch separates the two thresholds. The apex is a vertex rather than
/// a short edge, so nothing reverses; the mitre there runs away and bevels,
/// which inserts an extra point between the two walls; and once the walls
/// are no longer adjacent, `self_intersects` can see them cross. So the
/// "classic dart/notch failure" the variant documents is real, and it is
/// specifically the dart, not the notch, that produces it.
#[test]
fn a_dart_apex_reaches_the_self_intersection_branch() {
    let notched = PatternBoundary::new(vec![
        Point2::new(0.0, 0.0),
        Point2::new(100.0, 0.0),
        Point2::new(100.0, 100.0),
        Point2::new(52.0, 100.0),
        Point2::new(50.0, 80.0),
        Point2::new(48.0, 100.0),
        Point2::new(0.0, 100.0),
    ])
    .expect("valid");

    let err = notched.offset(1.0).unwrap_err();
    let GeometryError::OffsetSelfIntersects { edges, distance_mm } = err else {
        panic!("expected a crossing, got {err:?}");
    };
    assert_eq!(distance_mm, 1.0);
    assert_eq!(edges, (3, 5));
}
