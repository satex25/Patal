//! Millimetres and PostScript points, as two types that cannot be confused.
//!
//! This module is small and it is the highest-risk code in the crate. The
//! model is in millimetres. PDF user space is in points, defined as 1/72
//! inch. One multiplication separates them, and getting it wrong — or
//! applying it twice, or not at all — produces a document that opens, prints,
//! looks entirely plausible, and is the wrong size. No test that stays inside
//! the program can catch that; only a steel rule can.
//!
//! So the conversion is not a bare `f64` multiply anyone can write at a call
//! site. It happens in exactly one place, [`Mm::to_pt`], and everything
//! downstream carries a [`Pt`] that says what it is.

use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

/// Millimetres per PostScript point: 25.4 mm to the inch, 72 points to the
/// inch.
///
/// Exact in decimal, not exact in binary floating point, which is why the
/// tests assert the round trip against an independently computed constant
/// rather than against this one.
pub const MM_PER_PT: f64 = 25.4 / 72.0;

/// The reciprocal, written out rather than computed as `1.0 / MM_PER_PT`.
///
/// Dividing by [`MM_PER_PT`] and multiplying by this differ in the last unit
/// in the last place. Neither is more correct; having both spelled from the
/// same two exact integers keeps the discrepancy at one ulp instead of
/// letting it compound through a chain of derived constants.
pub const PT_PER_MM: f64 = 72.0 / 25.4;

/// A length in millimetres — the unit the model, the seam allowance, and the
/// designer all speak.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Mm(pub f64);

/// A length in PostScript points — the unit the page speaks.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Pt(pub f64);

impl Mm {
    /// The only conversion from model space to page space in this crate.
    ///
    /// It is not `const` on purpose. A `const fn` would be inlined into call
    /// sites as a bare multiply and stop showing up in a search for where
    /// units change.
    pub fn to_pt(self) -> Pt {
        Pt(self.0 * PT_PER_MM)
    }

    pub fn get(self) -> f64 {
        self.0
    }

    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }

    pub fn min(self, other: Self) -> Self {
        Mm(self.0.min(other.0))
    }

    pub fn max(self, other: Self) -> Self {
        Mm(self.0.max(other.0))
    }
}

impl Pt {
    pub fn to_mm(self) -> Mm {
        Mm(self.0 * MM_PER_PT)
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

macro_rules! arithmetic {
    ($t:ident) => {
        impl Add for $t {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                $t(self.0 + rhs.0)
            }
        }
        impl Sub for $t {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                $t(self.0 - rhs.0)
            }
        }
        /// Scaling by a dimensionless factor. Deliberately the only
        /// multiplication defined: `Mm * Mm` would be an area, and this crate
        /// has no use for one, so it stays unrepresentable.
        impl Mul<f64> for $t {
            type Output = Self;
            fn mul(self, rhs: f64) -> Self {
                $t(self.0 * rhs)
            }
        }
        impl Neg for $t {
            type Output = Self;
            fn neg(self) -> Self {
                $t(-self.0)
            }
        }
        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{}", self.0, stringify!($t).to_lowercase())
            }
        }
    };
}

arithmetic!(Mm);
arithmetic!(Pt);

#[cfg(test)]
mod tests {
    use super::*;

    /// The number the whole crate rests on, computed here from the two
    /// integers that define it and nowhere near the code under test.
    ///
    /// 50 mm is the calibration square. If this assertion and a steel rule
    /// ever disagree, the bug is downstream of this line, which is the point
    /// of pinning it.
    #[test]
    fn fifty_millimetres_is_one_hundred_and_forty_one_point_seven_three_points() {
        let independent = 50.0 * 72.0 / 25.4; // 141.73228346456693
        assert!(
            (Mm(50.0).to_pt().get() - independent).abs() < 1e-9,
            "{} vs {independent}",
            Mm(50.0).to_pt().get()
        );
        assert!((independent - 141.732_283_464_566_93).abs() < 1e-9);
    }

    #[test]
    fn one_millimetre_is_two_point_eight_three_points() {
        assert!((Mm(1.0).to_pt().get() - 2.834_645_669_291_338_6).abs() < 1e-12);
    }

    #[test]
    fn a4_is_the_size_everyone_quotes() {
        // 210 x 297 mm is 595.28 x 841.89 pt, the numbers every PDF tool
        // prints for A4. A MediaBox that disagrees is the first sign the
        // conversion is wrong.
        assert!((Mm(210.0).to_pt().get() - 595.275_590_551_181_1).abs() < 1e-9);
        assert!((Mm(297.0).to_pt().get() - 841.889_763_779_527_5).abs() < 1e-9);
    }

    #[test]
    fn the_two_constants_are_reciprocal() {
        assert!((MM_PER_PT * PT_PER_MM - 1.0).abs() < 1e-15);
    }

    #[test]
    fn round_tripping_a_length_returns_it() {
        for mm in [0.0, 0.4, 1.0, 50.0, 200.0, 297.0, 1000.0] {
            let back = Mm(mm).to_pt().to_mm().get();
            assert!((back - mm).abs() < 1e-9, "{mm} came back as {back}");
        }
    }
}
