use serde::{Deserialize, Serialize};
use crate::Point2;

/// Primitive curve segments
///
/// All coordinates in font space (Y-up, font units).
/// Segments are explicit: no implied points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Segment {
    /// Move to a point (starts new subpath)
    MoveTo {
        x: f64,
        y: f64,
    },

    /// Straight line to a point
    LineTo {
        x: f64,
        y: f64,
    },

    /// Quadratic Bézier (one control point)
    QuadTo {
        cx: f64,
        cy: f64,
        x: f64,
        y: f64,
    },

    /// Cubic Bézier (two control points)
    CubicTo {
        cx1: f64,
        cy1: f64,
        cx2: f64,
        cy2: f64,
        x: f64,
        y: f64,
    },
}

impl Segment {
    /// Get the endpoint of this segment (if it has one)
    pub fn endpoint(&self) -> Option<Point2> {
        match self {
            Segment::MoveTo { x, y } => Some(Point2::new(*x, *y)),
            Segment::LineTo { x, y } => Some(Point2::new(*x, *y)),
            Segment::QuadTo { x, y, .. } => Some(Point2::new(*x, *y)),
            Segment::CubicTo { x, y, .. } => Some(Point2::new(*x, *y)),
        }
    }

    /// Elevate a quadratic segment to cubic (exact, no approximation)
    ///
    /// Given quadratic Q(t) = (1-t)²P₀ + 2(1-t)tP₁ + t²P₂
    /// Convert to cubic C(t) with control points:
    /// - C₀ = P₀
    /// - C₁ = P₀ + ⅔(P₁ - P₀)
    /// - C₂ = P₂ + ⅔(P₁ - P₂)
    /// - C₃ = P₂
    pub fn elevate_quad_to_cubic(start: Point2, cx: f64, cy: f64, x: f64, y: f64) -> Segment {
        // Quadratic control point
        let p1 = Point2::new(cx, cy);
        // Quadratic endpoint
        let p2 = Point2::new(x, y);

        // Cubic control points
        let c1 = Point2::new(
            start.x + (2.0 / 3.0) * (p1.x - start.x),
            start.y + (2.0 / 3.0) * (p1.y - start.y),
        );
        let c2 = Point2::new(
            p2.x + (2.0 / 3.0) * (p1.x - p2.x),
            p2.y + (2.0 / 3.0) * (p1.y - p2.y),
        );

        Segment::CubicTo {
            cx1: c1.x,
            cy1: c1.y,
            cx2: c2.x,
            cy2: c2.y,
            x: p2.x,
            y: p2.y,
        }
    }
}
