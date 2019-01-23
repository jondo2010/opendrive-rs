use std::ops::Range;

pub type Length = euclid::Length<f64, euclid::UnknownUnit>; //, super::units::Meter>;
pub type Angle = euclid::Angle<f64>;

pub type Rotation = euclid::TypedRotation2D<f64, euclid::UnknownUnit, euclid::UnknownUnit>; //, super::units::Meter, super::units::Meter>;

#[derive(Copy, Clone)]
pub enum Segment<S> {
    Line(lyon_geom::LineSegment<S>),
    Quadratic(lyon_geom::QuadraticBezierSegment<S>),
    Cubic(lyon_geom::CubicBezierSegment<S>),
    Arc(lyon_geom::Arc<S>),
}

/// Forward all trait functions through to the underlying types
impl lyon_geom::Segment for Segment<f64> {
    type Scalar = f64;

    //fn from(&self) -> euclid::Point<Self::Scalar> {
    fn from(&self) -> euclid::Point2D<Self::Scalar> {
        match self {
            Segment::Line(ref line) => line.from(),
            Segment::Quadratic(ref quad) => quad.from(),
            Segment::Cubic(ref cub) => cub.from(),
            Segment::Arc(ref arc) => arc.from(),
        }
    }
    fn to(&self) -> euclid::Point2D<Self::Scalar> {
        match self {
            Segment::Line(ref line) => line.to(),
            Segment::Quadratic(ref quad) => quad.to(),
            Segment::Cubic(ref cub) => cub.to(),
            Segment::Arc(ref arc) => arc.to(),
        }
    }
    fn sample(&self, t: Self::Scalar) -> euclid::Point2D<Self::Scalar> {
        match self {
            Segment::Line(ref line) => line.sample(t),
            Segment::Quadratic(ref quad) => quad.sample(t),
            Segment::Cubic(ref cub) => cub.sample(t),
            Segment::Arc(ref arc) => arc.sample(t),
        }
    }
    fn x(&self, t: Self::Scalar) -> Self::Scalar {
        match self {
            Segment::Line(ref line) => line.x(t),
            Segment::Quadratic(ref quad) => quad.x(t),
            Segment::Cubic(ref cub) => cub.x(t),
            Segment::Arc(ref arc) => arc.x(t),
        }
    }
    fn y(&self, t: Self::Scalar) -> Self::Scalar {
        match self {
            Segment::Line(ref line) => line.y(t),
            Segment::Quadratic(ref quad) => quad.y(t),
            Segment::Cubic(ref cub) => cub.y(t),
            Segment::Arc(ref arc) => arc.y(t),
        }
    }
    fn derivative(&self, _t: Self::Scalar) -> euclid::Vector2D<Self::Scalar> {
        match self {
            Segment::Line(ref line) => line.derivative(_t),
            Segment::Quadratic(ref quad) => quad.derivative(_t),
            Segment::Cubic(ref cub) => cub.derivative(_t),
            Segment::Arc(ref arc) => arc.derivative(_t),
        }
    }
    fn dx(&self, _t: Self::Scalar) -> Self::Scalar {
        match self {
            Segment::Line(ref line) => line.dx(_t),
            Segment::Quadratic(ref quad) => quad.dx(_t),
            Segment::Cubic(ref cub) => cub.dx(_t),
            Segment::Arc(ref arc) => arc.dx(_t),
        }
    }
    fn dy(&self, _t: Self::Scalar) -> Self::Scalar {
        match self {
            Segment::Line(ref line) => line.dy(_t),
            Segment::Quadratic(ref quad) => quad.dy(_t),
            Segment::Cubic(ref cub) => cub.dy(_t),
            Segment::Arc(ref arc) => arc.dy(_t),
        }
    }
    fn split_range(&self, t_range: Range<Self::Scalar>) -> Self {
        match self {
            Segment::Line(ref line) => Segment::Line(line.split_range(t_range)),
            Segment::Quadratic(ref quad) => Segment::Quadratic(quad.split_range(t_range)),
            Segment::Cubic(ref cub) => Segment::Cubic(cub.split_range(t_range)),
            Segment::Arc(ref arc) => Segment::Arc(arc.split_range(t_range)),
        }
    }
    fn split(&self, t: Self::Scalar) -> (Self, Self) {
        match self {
            Segment::Line(ref line) => {
                let (a, b) = line.split(t);
                (Segment::Line(a), Segment::Line(b))
            }
            Segment::Quadratic(ref quad) => {
                let (a, b) = quad.split(t);
                (Segment::Quadratic(a), Segment::Quadratic(b))
            }
            Segment::Cubic(ref cub) => {
                let (a, b) = cub.split(t);
                (Segment::Cubic(a), Segment::Cubic(b))
            }
            Segment::Arc(ref arc) => {
                let (a, b) = arc.split(t);
                (Segment::Arc(a), Segment::Arc(b))
            }
        }
    }
    fn before_split(&self, t: Self::Scalar) -> Self {
        match self {
            Segment::Line(ref line) => Segment::Line(line.before_split(t)),
            Segment::Quadratic(ref quad) => Segment::Quadratic(quad.before_split(t)),
            Segment::Cubic(ref cub) => Segment::Cubic(cub.before_split(t)),
            Segment::Arc(ref arc) => Segment::Arc(arc.before_split(t)),
        }
    }
    fn after_split(&self, t: Self::Scalar) -> Self {
        match self {
            Segment::Line(ref line) => Segment::Line(line.before_split(t)),
            Segment::Quadratic(ref quad) => Segment::Quadratic(quad.after_split(t)),
            Segment::Cubic(ref cub) => Segment::Cubic(cub.after_split(t)),
            Segment::Arc(ref arc) => Segment::Arc(arc.after_split(t)),
        }
    }
    fn flip(&self) -> Self {
        match self {
            Segment::Line(ref line) => Segment::Line(line.flip()),
            Segment::Quadratic(ref quad) => Segment::Quadratic(quad.flip()),
            Segment::Cubic(ref cub) => Segment::Cubic(cub.flip()),
            Segment::Arc(ref arc) => Segment::Arc(arc.flip()),
        }
    }
    fn approximate_length(&self, _tolerance: Self::Scalar) -> Self::Scalar {
        match self {
            Segment::Line(ref line) => line.approximate_length(_tolerance),
            Segment::Quadratic(ref quad) => quad.approximate_length(_tolerance),
            Segment::Cubic(ref cub) => cub.approximate_length(_tolerance),
            Segment::Arc(ref arc) => arc.approximate_length(_tolerance),
        }
    }
}
