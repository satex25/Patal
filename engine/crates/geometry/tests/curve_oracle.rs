//! The specification for curve flattening, stated as a closed-form oracle.
//!
//! A circle is the one shape whose exact offset is known analytically:
//! offsetting radius `R` by `d` gives exactly `R + d`, everywhere. So a
//! circle approximated by cubics, flattened, and offset can be checked
//! against arithmetic rather than against a previous run of the same code.
//!
//! The sweep deliberately covers all four combinations of curvature sign
//! and offset direction. An earlier version of this plan tested only
//! positive `d` on circles, which are convex everywhere — so it never
//! exercised an inset and never exercised a concave region, which are
//! exactly the two cases where the tolerance-scaling argument is not
//! trivially true:
//!
//! | | outset (`d > 0`) | inset (`d < 0`) |
//! |---|---|---|
//! | convex (circle) | `circle_oracle_sweep` | `circle_oracle_sweep` |
//! | concave (bite) | `concave_arc_outset_stays_within_tolerance` | `concave_arc_inset_stays_within_tolerance` |

use patal_geometry::{
    Edge, EdgeSegment, GeometryError, Join, PatternBoundary, Point2, SeamPath, Winding,
};

/// One cubic approximating a circular arc from `theta0` to `theta1`.
///
/// The `4/3·tan(sweep/4)` control-point offset is the standard best
/// approximation; for a quarter turn it is the familiar 0.5522847498. The
/// signed sweep makes this work in both directions without a special case:
/// a negative sweep produces a negative `k`, which flips the tangents.
fn arc_cubic(centre: Point2, radius: f64, theta0: f64, theta1: f64) -> (Point2, EdgeSegment) {
    let sweep = theta1 - theta0;
    let k = (4.0 / 3.0) * (sweep / 4.0).tan();

    let on = |t: f64| Point2::new(centre.x + radius * t.cos(), centre.y + radius * t.sin());
    let tangent = |t: f64| Point2::new(-t.sin(), t.cos());

    let p0 = on(theta0);
    let p3 = on(theta1);
    let t0 = tangent(theta0);
    let t1 = tangent(theta1);

    let c1 = Point2::new(p0.x + k * radius * t0.x, p0.y + k * radius * t0.y);
    let c2 = Point2::new(p3.x - k * radius * t1.x, p3.y - k * radius * t1.y);

    (p0, EdgeSegment::Cubic { c1, c2, to: p3 })
}

/// How many cubics the oracle's circle is built from.
///
/// **Not four**, which is what the plan specified, and the difference is the
/// whole reason the first version of this sweep failed.
///
/// A cubic Bézier cannot represent a circular arc exactly. A quarter-arc
/// approximation is off by about 2.7e-4 of the radius — a real, documented
/// property of the construction, pinned below in
/// `four_cubics_are_not_accurate_enough_to_be_an_oracle`. On a 1000mm circle
/// that is 0.27mm of error *in the reference shape*, 270 times the 0.001mm
/// tolerance the sweep is trying to verify. A test built that way measures
/// how badly cubics approximate circles, not how well flattening works, and
/// it cannot pass at a tight tolerance no matter how correct the code is.
///
/// The error falls as roughly the sixth power of the arc angle, so 32
/// segments bring it to ~1e-9 of the radius: a picometre on a 1000mm piece,
/// which is genuinely negligible against everything else being measured.
const ORACLE_ARCS: usize = 32;

/// A full circle as `arcs` cubic segments, wound counter-clockwise.
fn circle_with(centre: Point2, radius: f64, arcs: usize) -> SeamPath {
    let step = std::f64::consts::TAU / arcs as f64;
    let mut segments = Vec::with_capacity(arcs);
    let mut start = None;

    for i in 0..arcs {
        let (p0, segment) = arc_cubic(centre, radius, i as f64 * step, (i + 1) as f64 * step);
        if start.is_none() {
            start = Some(p0);
        }
        segments.push(segment);
    }

    // `closed` rather than `new`: the last arc ends at sin(2π), which is
    // -2.4e-16 rather than 0, so a generated circle misses its own start by
    // float noise. Snapping that is exactly what `closed` is for.
    SeamPath::closed(start.expect("at least one arc"), segments).expect("a circle is a closed path")
}

fn circle(centre: Point2, radius: f64) -> SeamPath {
    circle_with(centre, radius, ORACLE_ARCS)
}

/// The measurement that decides how the sweep above has to be built.
///
/// Sampling the flattened outline at a tolerance far finer than the effect
/// being measured isolates the arc construction's own error. Quarter-arcs
/// land near 2.7e-4 of the radius; 32 arcs land near 1e-9. If this ever
/// drifts, the oracle's error budget has moved and the sweep's assertions
/// need revisiting before they are trusted.
#[test]
fn four_cubics_are_not_accurate_enough_to_be_an_oracle() {
    let centre = Point2::new(0.0, 0.0);
    let radius = 100.0;

    let worst_relative_error = |arcs: usize| {
        circle_with(centre, radius, arcs)
            .flatten(1e-9)
            .unwrap()
            .points()
            .iter()
            .map(|p| (distance_from(centre, p) - radius).abs() / radius)
            .fold(0.0f64, f64::max)
    };

    let quarters = worst_relative_error(4);
    assert!(
        (1e-4..1e-3).contains(&quarters),
        "quarter-arc error moved: {quarters:e}"
    );

    let fine = worst_relative_error(ORACLE_ARCS);
    assert!(
        fine < 1e-8,
        "oracle circle is no longer negligible: {fine:e}"
    );
}

fn distance_from(centre: Point2, p: &Point2) -> f64 {
    centre.distance(p)
}

// ---------------------------------------------------------------------
// Convex, both directions
// ---------------------------------------------------------------------

#[test]
fn circle_oracle_sweep() {
    let centre = Point2::new(0.0, 0.0);

    for radius in [10.0f64, 50.0, 200.0, 1000.0] {
        for distance in [1.0f64, 5.0, 10.0, 25.0, -1.0, -5.0, -10.0, -25.0] {
            // An inset deeper than the radius has no answer; that case is
            // covered by `an_inset_past_the_radius_fails_loudly`.
            if distance <= -radius {
                continue;
            }

            for tolerance in [0.1f64, 0.01, 0.001] {
                let path = circle(centre, radius);
                let flattened = path
                    .flatten_for_offset(tolerance, distance)
                    .unwrap_or_else(|e| panic!("R={radius} d={distance} t={tolerance}: {e}"));

                let cut = flattened
                    .offset(distance)
                    .unwrap_or_else(|e| panic!("R={radius} d={distance} t={tolerance}: {e}"));

                let expected = radius + distance;
                for point in cut.points() {
                    let actual = distance_from(centre, point);
                    assert!(
                        (actual - expected).abs() <= tolerance,
                        "R={radius} d={distance} t={tolerance}: point at {actual} \
                         deviates {} from the analytic offset radius {expected}",
                        (actual - expected).abs()
                    );
                }
            }
        }
    }
}

#[test]
fn flattened_perimeter_approaches_the_circumference_from_below() {
    // Chords are shorter than the arcs they replace, so a flattened circle
    // is always slightly short — and gets less short as tolerance tightens.
    // A flattening that ever overshot would mean vertices off the curve.
    let radius = 100.0;
    let circumference = 2.0 * std::f64::consts::PI * radius;
    let mut previous = 0.0;

    for tolerance in [1.0f64, 0.1, 0.01, 0.001] {
        let perimeter = circle(Point2::new(0.0, 0.0), radius)
            .flatten(tolerance)
            .unwrap()
            .perimeter();

        assert!(
            perimeter < circumference,
            "t={tolerance}: {perimeter} is not below {circumference}"
        );
        assert!(
            perimeter > previous,
            "t={tolerance}: tightening the tolerance made the perimeter worse"
        );
        previous = perimeter;
    }

    assert!((previous - circumference).abs() < 0.01);
}

#[test]
fn a_flattened_circle_is_wound_the_way_it_was_drawn() {
    let flattened = circle(Point2::new(0.0, 0.0), 100.0).flatten(0.01).unwrap();
    assert_eq!(flattened.winding(), Winding::CounterClockwise);
}

#[test]
fn tightening_the_tolerance_costs_vertices_and_buys_accuracy() {
    let centre = Point2::new(0.0, 0.0);
    let radius = 100.0;

    // Quarter-arcs, not the 32-arc oracle circle: with 32 arcs each cubic
    // is already flat enough that both tolerances terminate immediately at
    // the same vertex count, and the test would assert nothing.
    let coarse = circle_with(centre, radius, 4).flatten(1.0).unwrap();
    let fine = circle_with(centre, radius, 4).flatten(0.01).unwrap();

    assert!(
        fine.points().len() > coarse.points().len(),
        "coarse {} vs fine {}",
        coarse.points().len(),
        fine.points().len()
    );

    let worst = |b: &PatternBoundary| {
        b.points()
            .iter()
            .map(|p| (distance_from(centre, p) - radius).abs())
            .fold(0.0f64, f64::max)
    };
    // Vertices land on the authored curve either way — subdivision only
    // ever evaluates the curve, it never moves a point off it — so what a
    // tighter tolerance buys is chords that sag less, not vertices that sit
    // more accurately. Both are bounded by the quarter-arc construction's
    // own ~2.7e-4 relative error, and neither improves with tolerance.
    let quarter_arc_error = radius * 1e-3;
    assert!(worst(&coarse) < quarter_arc_error);
    assert!(worst(&fine) < quarter_arc_error);
}

// ---------------------------------------------------------------------
// Concave, both directions
// ---------------------------------------------------------------------

/// A 200mm square with a semicircular bite taken out of its top edge.
///
/// The bite arc curves *into* the material, so it is genuinely concave —
/// the case a circle can never produce, because a circle is convex
/// everywhere no matter which way it is wound.
fn square_with_a_bite(bite_radius: f64) -> (SeamPath, Point2) {
    let bite_centre = Point2::new(100.0, 200.0);
    let start = Point2::new(0.0, 0.0);

    let (_, first_half) = arc_cubic(bite_centre, bite_radius, 0.0, -std::f64::consts::FRAC_PI_2);
    let (_, second_half) = arc_cubic(
        bite_centre,
        bite_radius,
        -std::f64::consts::FRAC_PI_2,
        -std::f64::consts::PI,
    );

    let path = SeamPath::new(
        start,
        vec![
            EdgeSegment::Line {
                to: Point2::new(200.0, 0.0),
            },
            EdgeSegment::Line {
                to: Point2::new(200.0, 200.0),
            },
            // Right lip of the bite.
            EdgeSegment::Line {
                to: Point2::new(100.0 + bite_radius, 200.0),
            },
            first_half,
            second_half,
            // Left lip.
            EdgeSegment::Line {
                to: Point2::new(0.0, 200.0),
            },
            EdgeSegment::Line { to: start },
        ],
    )
    .expect("closed");

    (path, bite_centre)
}

/// Points that ended up on the offset arc, identified by being no further
/// from the bite centre than the arc itself (everything else on the piece
/// is further away). Returned so the caller can assert the set is not
/// empty — an oracle that silently checks nothing is worse than none.
fn points_on_the_bite(
    boundary: &PatternBoundary,
    bite_centre: Point2,
    arc_radius: f64,
    tolerance: f64,
) -> Vec<f64> {
    boundary
        .points()
        .iter()
        .map(|p| distance_from(bite_centre, p))
        .filter(|d| *d <= arc_radius + tolerance)
        .collect()
}

#[test]
fn concave_arc_inset_stays_within_tolerance() {
    // The amplifying combination. Insetting a concave arc moves it *away*
    // from its own centre, so the radius grows from r to r + |d| and the
    // flattening sag grows with it. This is the case the old sweep could
    // not reach, and the one where under-tightening would show up.
    let bite_radius = 50.0;

    // Tolerance and allowance are paired here rather than crossed, and the
    // pairing is load-bearing. A fine tolerance produces chords shorter than
    // a coarse allowance, which this shape's square lip corner cannot
    // survive — see
    // `a_chord_shorter_than_the_allowance_at_a_sharp_corner_collapses`,
    // which pins that interaction on purpose. Crossing the two here would
    // trip over it by accident instead of measuring the amplification this
    // test exists for.
    for (tolerance, inset) in [
        (0.1f64, -4.0f64),
        (0.1, -1.0),
        (0.01, -1.0),
        (0.01, -0.5),
        (0.001, -0.2),
    ] {
        let (path, bite_centre) = square_with_a_bite(bite_radius);
        let flattened = path.flatten_for_offset(tolerance, inset).unwrap();
        let cut = flattened
            .offset(inset)
            .unwrap_or_else(|e| panic!("bite inset by {inset} at t={tolerance}: {e}"));

        let expected = bite_radius + inset.abs();
        let on_arc = points_on_the_bite(&cut, bite_centre, expected, tolerance);
        assert!(
            on_arc.len() > 8,
            "d={inset} t={tolerance}: only {} points landed on the arc, \
             the assertion below would be vacuous",
            on_arc.len()
        );

        for actual in on_arc {
            assert!(
                (actual - expected).abs() <= tolerance,
                "d={inset} t={tolerance}: arc point at {actual} deviates {} \
                 from the analytic inset radius {expected}",
                (actual - expected).abs()
            );
        }
    }
}

/// The interaction the two-layer design did not anticipate, pinned so it
/// cannot be rediscovered as a mystery.
///
/// Flattening finely enough for accuracy can make a shape *un-offsettable*,
/// even though the shape it approximates has an exact analytic offset.
///
/// The bite meets the top edge at a square corner. Insetting by `d` consumes
/// `d·tan(θ/2)` of length from each edge adjacent to a corner of turn `θ`,
/// which at 90° is `d` exactly. Tighten the tolerance and the arc's first
/// chord gets shorter; once it is shorter than the allowance, that edge
/// reverses and the kernel correctly refuses. Below, the same shape and the
/// same 0.5mm allowance succeed at a 0.01mm tolerance and fail at 0.001mm.
///
/// The kernel is not wrong — the polygon it was handed really does collapse.
/// The lesson is about the layer above: tolerance cannot be chosen purely
/// for accuracy, because it also sets a floor on the seam allowance a piece
/// can carry at a sharp corner. Recorded in ADR-003 as work the curve layer
/// still owes.
#[test]
fn a_chord_shorter_than_the_allowance_at_a_sharp_corner_collapses() {
    let allowance = -0.5;

    let (coarse, _) = square_with_a_bite(50.0);
    let coarse = coarse.flatten_for_offset(0.01, allowance).unwrap();
    assert!(
        coarse.offset(allowance).is_ok(),
        "a 0.5mm allowance fits when chords are ~1.2mm"
    );

    let (fine, _) = square_with_a_bite(50.0);
    let fine = fine.flatten_for_offset(0.001, allowance).unwrap();

    let shortest = shortest_edge(&fine);
    assert!(shortest < allowance.abs(), "shortest chord is {shortest}");
    assert!(
        matches!(
            fine.offset(allowance),
            Err(GeometryError::OffsetCollapsed { .. })
        ),
        "the same allowance must be refused once a chord is shorter than it"
    );
}

fn shortest_edge(boundary: &PatternBoundary) -> f64 {
    let points = boundary.points();
    let n = points.len();
    (0..n)
        .map(|i| points[i].distance(&points[(i + 1) % n]))
        .fold(f64::MAX, f64::min)
}

#[test]
fn concave_arc_outset_stays_within_tolerance() {
    // The shrinking combination, included so the table is complete rather
    // than because it is at risk: offsetting a concave arc outward moves it
    // toward its own centre, so r shrinks to r - d and the sag shrinks too.
    let bite_radius = 50.0;

    // Paired for the same reason as the inset case above.
    for (tolerance, outset) in [
        (0.1f64, 4.0f64),
        (0.1, 1.0),
        (0.01, 1.0),
        (0.01, 0.5),
        (0.001, 0.2),
    ] {
        let (path, bite_centre) = square_with_a_bite(bite_radius);
        let flattened = path.flatten_for_offset(tolerance, outset).unwrap();
        let cut = flattened
            .offset(outset)
            .unwrap_or_else(|e| panic!("bite outset by {outset} at t={tolerance}: {e}"));

        let expected = bite_radius - outset;
        let on_arc = points_on_the_bite(&cut, bite_centre, expected, tolerance);
        assert!(on_arc.len() > 8, "d={outset} t={tolerance}: vacuous");

        for actual in on_arc {
            assert!(
                (actual - expected).abs() <= tolerance,
                "d={outset} t={tolerance}: arc point at {actual} deviates {} \
                 from the analytic outset radius {expected}",
                (actual - expected).abs()
            );
        }
    }
}

#[test]
fn an_inset_past_the_radius_fails_loudly() {
    // |d|·κ >= 1: the seam allowance exceeds what the curvature can give.
    // The right answer is a refusal, not a plausible-looking polygon — and
    // the message names the edges that cross, which is what lets a UI point
    // at the place the designer has to change.
    let path = circle(Point2::new(0.0, 0.0), 20.0);
    let flattened = path.flatten_for_offset(0.01, -25.0).unwrap();

    match flattened.offset(-25.0) {
        Err(GeometryError::OffsetCollapsed { .. })
        | Err(GeometryError::OffsetSelfIntersects { .. }) => {}
        other => panic!("an inset past the radius must fail loudly, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// Construction contract
// ---------------------------------------------------------------------

#[test]
fn a_path_that_does_not_close_is_rejected() {
    let start = Point2::new(0.0, 0.0);
    let err = SeamPath::new(
        start,
        vec![
            EdgeSegment::Line {
                to: Point2::new(10.0, 0.0),
            },
            EdgeSegment::Line {
                to: Point2::new(10.0, 10.0),
            },
        ],
    )
    .unwrap_err();

    assert!(matches!(err, GeometryError::PathNotClosed { .. }));
    assert!(err.to_string().contains("ends at (10, 10)"));
}

#[test]
fn closed_appends_the_edge_new_refuses_to_invent() {
    let start = Point2::new(0.0, 0.0);
    let open = vec![
        EdgeSegment::Line {
            to: Point2::new(10.0, 0.0),
        },
        EdgeSegment::Line {
            to: Point2::new(10.0, 10.0),
        },
    ];

    let path = SeamPath::closed(start, open.clone()).expect("closes");
    assert_eq!(path.edges().len(), open.len() + 1);
    assert_eq!(path.edges().last().unwrap().end(), start);

    // An already-closed path is left exactly as it was.
    let rebuilt: Vec<EdgeSegment> = path.edges().iter().map(|e| e.geometry()).collect();
    let already = SeamPath::closed(start, rebuilt).unwrap();
    assert_eq!(already.edges().len(), path.edges().len());
}

#[test]
fn a_non_finite_control_point_names_its_segment() {
    let start = Point2::new(0.0, 0.0);
    let err = SeamPath::new(
        start,
        vec![
            EdgeSegment::Line {
                to: Point2::new(10.0, 0.0),
            },
            EdgeSegment::Cubic {
                c1: Point2::new(f64::NAN, 0.0),
                c2: Point2::new(5.0, 5.0),
                to: start,
            },
        ],
    )
    .unwrap_err();

    assert_eq!(err, GeometryError::NonFiniteControlPoint { segment: 1 });
}

#[test]
fn a_non_positive_tolerance_is_rejected() {
    let path = circle(Point2::new(0.0, 0.0), 50.0);
    for tolerance in [0.0f64, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            matches!(
                path.flatten(tolerance),
                Err(GeometryError::ToleranceNotPositive { .. })
            ),
            "tolerance {tolerance} should have been rejected"
        );
    }
}

#[test]
fn a_seam_path_round_trips_through_json_via_its_constructor() {
    let path = circle(Point2::new(3.0, -4.0), 25.0);
    let json = serde_json::to_string(&path).unwrap();
    let restored: SeamPath = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, path);
}

#[test]
fn a_hand_edited_open_path_cannot_be_deserialized() {
    // The same guarantee PatternBoundary has: validation is not something
    // a file can skip by being written by hand.
    let json = r#"{"start":{"x":0.0,"y":0.0},
                   "edges":[{"geometry":{"kind":"line","to":{"x":10.0,"y":0.0}}}]}"#;
    let err = serde_json::from_str::<SeamPath>(json).unwrap_err();
    assert!(err.to_string().contains("ends at"), "{err}");
}

#[test]
fn a_straight_path_needs_no_subdivision_and_no_curvature() {
    let start = Point2::new(0.0, 0.0);
    let path = SeamPath::closed(
        start,
        vec![
            EdgeSegment::Line {
                to: Point2::new(100.0, 0.0),
            },
            EdgeSegment::Line {
                to: Point2::new(100.0, 100.0),
            },
            EdgeSegment::Line {
                to: Point2::new(0.0, 100.0),
            },
        ],
    )
    .unwrap();

    assert_eq!(path.max_curvature(), 0.0);
    let flattened = path.flatten(0.001).unwrap();
    assert_eq!(flattened.points().len(), 4);

    // With zero curvature there is nothing to tighten for, so a huge
    // offset must not change the result.
    let for_offset = path.flatten_for_offset(0.001, 500.0).unwrap();
    assert_eq!(for_offset.points(), flattened.points());
}

#[test]
fn every_edge_from_the_plain_constructor_is_a_corner() {
    let start = Point2::new(0.0, 0.0);
    let path = SeamPath::new(
        start,
        vec![
            EdgeSegment::Line {
                to: Point2::new(10.0, 0.0),
            },
            EdgeSegment::Line {
                to: Point2::new(10.0, 10.0),
            },
            EdgeSegment::Line { to: start },
        ],
    )
    .expect("a triangle closes");

    assert_eq!(path.edges().len(), 3);
    assert!(path.edges().iter().all(|e| e.join() == Join::Corner));
    assert_eq!(
        path.edges()[0].geometry(),
        EdgeSegment::Line {
            to: Point2::new(10.0, 0.0)
        }
    );
    assert_eq!(path.edges()[2].end(), start);
}

#[test]
fn an_edge_is_a_nested_object_on_the_wire_not_a_flat_one() {
    // The nesting is the point. When per-edge allowance (P-03) and fold
    // edges (P-05) arrive they are siblings of `join`, while `to` and `c1`
    // are the geometry itself. A flat map puts them in one bag as though
    // they were the same kind of thing, and the shape stops teaching the
    // distinction the container was chosen to make.
    let start = Point2::new(0.0, 0.0);
    let path = SeamPath::new(
        start,
        vec![
            EdgeSegment::Line {
                to: Point2::new(1.0, 0.0),
            },
            EdgeSegment::Line { to: start },
        ],
    )
    .expect("closes");

    let json = serde_json::to_value(&path).expect("serializes");
    let edge = &json["edges"][0];
    assert_eq!(edge["geometry"]["kind"], "line");
    assert_eq!(edge["join"], "corner");
    assert!(
        edge.get("to").is_none(),
        "geometry must not be flattened into the edge: {edge}"
    );
}

#[test]
fn an_edge_with_no_join_key_loads_as_a_corner() {
    // `Corner` is the absence of a claim, so omitting it cannot manufacture
    // one. A hand-edited `.patal` is explicitly in scope for this repo.
    let json = r#"{
        "start": {"x": 0.0, "y": 0.0},
        "edges": [
            {"geometry": {"kind": "line", "to": {"x": 1.0, "y": 0.0}}},
            {"geometry": {"kind": "line", "to": {"x": 0.0, "y": 0.0}}}
        ]
    }"#;
    let path: SeamPath = serde_json::from_str(json).expect("loads without a join key");
    assert!(path.edges().iter().all(|e| e.join() == Join::Corner));
}

#[test]
fn an_edge_carries_the_join_it_was_built_with() {
    // The container's whole job: geometry and its attributes travel together.
    let edge = Edge::new(
        EdgeSegment::Line {
            to: Point2::new(5.0, 0.0),
        },
        Join::Smooth,
    );
    assert_eq!(edge.join(), Join::Smooth);
    assert_eq!(edge.end(), Point2::new(5.0, 0.0));
    assert_eq!(Edge::corner(edge.geometry()).join(), Join::Corner);
}
