use scalar::{cast, Float, Scalar};
use generic_math::{Point, point, Vector, vector, Rotation2D, Transform2D, Angle, Rect};

/// A clothoid segment
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Clothoid<S> {
    pub scale: S,
    pub arg_start: S,
    pub arg_end: S,
}

impl<S: Scalar> Clothoid<S> {
    pub fn from_opendrive(
        from: Point<S>,
        start_curv: S,
        end_curv: S,
        length: S,
        rot: Angle<S>,
    ) -> Clothoid<S> {
        debug_assert!(!from.x.is_nan());
        debug_assert!(!from.y.is_nan());

        let abs_k0 = start_curv.abs();
        let abs_k1 = end_curv.abs();
        let x = (-length / (abs_k0 - abs_k1)).sqrt();

        let a = -k1 * sqrt(-l / (Abs(k0) - Abs(k1))) / Abs(k1);
        let t0 = k1 * sqrt(-l / (Abs(k0) - Abs(k1))) * Abs(k0) / Abs(k1);
        let t1 = k1 * sqrt(-l / (Abs(k0) - Abs(k1)));
    }
}
