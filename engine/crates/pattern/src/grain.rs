//! The direction a piece is laid on the cloth.

use patal_geometry::Point2;
use serde::{Deserialize, Serialize};

use crate::PatternError;

/// The wire shape of a [`GrainLine`], so serde routes through the validator.
#[derive(Serialize, Deserialize)]
struct GrainLineData {
    angle_deg: f64,
    anchor: Point2,
}

/// Which way the warp runs through a piece, and where the marking sits.
///
/// The angle is normalised into `[0, 360)` — a full circle, not a half one.
/// Grain is **directional, not axial**: napped fabrics such as velvet and
/// corduroy shade differently depending on which way the pile lies, so every
/// piece must be laid the same way up. Folding 190° onto 10° would lose
/// exactly the constraint that makes such a garment cuttable.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "GrainLineData", into = "GrainLineData")]
pub struct GrainLine {
    angle_deg: f64,
    anchor: Point2,
}

impl GrainLine {
    pub fn new(angle_deg: f64, anchor: Point2) -> Result<Self, PatternError> {
        if !angle_deg.is_finite() {
            return Err(PatternError::InvalidGrainLine {
                field: "angle_deg",
                value: angle_deg,
            });
        }
        if !anchor.is_finite() {
            // Report whichever coordinate is at fault, so the message names a
            // number the caller can find in their file.
            let value = if anchor.x.is_finite() {
                anchor.y
            } else {
                anchor.x
            };
            return Err(PatternError::InvalidGrainLine {
                field: "anchor",
                value,
            });
        }

        // `rem_euclid` lands in [0, 360) for negative input too, which is the
        // whole reason it is used here rather than `%`.
        Ok(Self {
            angle_deg: angle_deg.rem_euclid(360.0),
            anchor,
        })
    }

    pub fn angle_deg(&self) -> f64 {
        self.angle_deg
    }

    pub fn anchor(&self) -> Point2 {
        self.anchor
    }
}

impl From<GrainLine> for GrainLineData {
    fn from(grain: GrainLine) -> Self {
        Self {
            angle_deg: grain.angle_deg,
            anchor: grain.anchor,
        }
    }
}

impl TryFrom<GrainLineData> for GrainLine {
    type Error = PatternError;

    fn try_from(data: GrainLineData) -> Result<Self, Self::Error> {
        Self::new(data.angle_deg, data.anchor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_angle_past_a_full_turn_comes_back_inside_one() {
        let grain = GrainLine::new(450.0, Point2::new(0.0, 0.0)).expect("valid");
        assert_eq!(grain.angle_deg(), 90.0);
    }

    #[test]
    fn a_negative_angle_normalises_forward_not_backward() {
        let grain = GrainLine::new(-170.0, Point2::new(0.0, 0.0)).expect("valid");
        assert_eq!(grain.angle_deg(), 190.0);
    }

    #[test]
    fn one_hundred_and_ninety_degrees_stays_one_hundred_and_ninety() {
        // The whole reason normalisation is [0,360) and not [0,180). A grain
        // line is directional, not axial: napped fabrics — velvet, corduroy —
        // require every piece laid the same way up, and folding 190 onto 10
        // would silently destroy that. It looks like a tidy simplification
        // right up until someone cuts a velvet jacket.
        let grain = GrainLine::new(190.0, Point2::new(0.0, 0.0)).expect("valid");
        assert_eq!(grain.angle_deg(), 190.0);
    }

    #[test]
    fn a_full_turn_is_zero_not_three_hundred_and_sixty() {
        let grain = GrainLine::new(360.0, Point2::new(0.0, 0.0)).expect("valid");
        assert_eq!(grain.angle_deg(), 0.0);
    }

    #[test]
    fn a_non_finite_angle_is_refused_rather_than_normalised_into_something_plausible() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let err = GrainLine::new(bad, Point2::new(0.0, 0.0)).expect_err("refused");
            assert!(matches!(
                err,
                PatternError::InvalidGrainLine {
                    field: "angle_deg",
                    ..
                }
            ));
        }
    }

    #[test]
    fn a_non_finite_anchor_is_refused() {
        let err = GrainLine::new(0.0, Point2::new(f64::NAN, 0.0)).expect_err("refused");
        assert!(matches!(
            err,
            PatternError::InvalidGrainLine {
                field: "anchor",
                ..
            }
        ));
    }

    #[test]
    fn a_hand_edited_grain_line_cannot_skip_the_check() {
        // C6: serde routes through the constructor.
        let json = r#"{"angle_deg": 1e400, "anchor": {"x": 0.0, "y": 0.0}}"#;
        assert!(serde_json::from_str::<GrainLine>(json).is_err());
    }
}
