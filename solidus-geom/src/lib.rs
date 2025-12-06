//! Solidus geometry primitives
//!
//! Shared types for font geometry pipeline:
//! - Point2: 2D point in font space
//! - Segment: primitive curve segments (MoveTo, LineTo, QuadTo, CubicTo)
//! - Contour: closed/open paths made of segments
//! - BSplineCurve: cubic non-rational B-splines

pub mod point;
pub mod segment;
pub mod contour;
pub mod bspline;

pub use point::Point2;
pub use segment::Segment;
pub use contour::Contour;
pub use bspline::{BSplineCurve, KnotVector};
