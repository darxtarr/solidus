use solidus_geom::{BSplineCurve, Contour, KnotVector, Point2, Segment};

/// Convert a contour of primitive segments to a cubic B-spline
///
/// For v0: Simple approach using exact degree elevation
/// - Lines become cubics (degenerate)
/// - Quadratics are elevated to cubics exactly
/// - Cubics stay cubics
/// - Merge into single B-spline per contour
pub fn contour_to_bspline(contour: &Contour) -> Result<BSplineCurve, String> {
    if contour.segments.is_empty() {
        return Err("Empty contour".to_string());
    }

    // Extract cubic segments (after elevation)
    let mut control_points = Vec::new();
    let mut current_point = Point2::new(0.0, 0.0);

    for segment in &contour.segments {
        match segment {
            Segment::MoveTo { x, y } => {
                current_point = Point2::new(*x, *y);
                // First control point
                if control_points.is_empty() {
                    control_points.push(current_point);
                }
            }

            Segment::LineTo { x, y } => {
                // Line: convert to degenerate cubic (all control points on line)
                let end = Point2::new(*x, *y);
                let c1 = current_point.lerp(&end, 1.0 / 3.0);
                let c2 = current_point.lerp(&end, 2.0 / 3.0);

                control_points.push(c1);
                control_points.push(c2);
                control_points.push(end);

                current_point = end;
            }

            Segment::QuadTo { cx, cy, x, y } => {
                // Quadratic: elevate to cubic exactly
                let cubic = Segment::elevate_quad_to_cubic(current_point, *cx, *cy, *x, *y);

                if let Segment::CubicTo {
                    cx1,
                    cy1,
                    cx2,
                    cy2,
                    x: ex,
                    y: ey,
                } = cubic
                {
                    control_points.push(Point2::new(cx1, cy1));
                    control_points.push(Point2::new(cx2, cy2));
                    control_points.push(Point2::new(ex, ey));
                    current_point = Point2::new(ex, ey);
                } else {
                    unreachable!("elevate_quad_to_cubic should always return CubicTo");
                }
            }

            Segment::CubicTo {
                cx1,
                cy1,
                cx2,
                cy2,
                x,
                y,
            } => {
                // Cubic: use directly
                control_points.push(Point2::new(*cx1, *cy1));
                control_points.push(Point2::new(*cx2, *cy2));
                control_points.push(Point2::new(*x, *y));
                current_point = Point2::new(*x, *y);
            }
        }
    }

    if control_points.is_empty() {
        return Err("No control points generated".to_string());
    }

    // Create knot vector (chord-length parameterization)
    let knots = KnotVector::clamped_chord_length(3, &control_points);

    // Create B-spline curve
    let curve = BSplineCurve::new_cubic(control_points, knots, contour.closed);

    // Validate
    curve.validate()?;

    Ok(curve)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_line_to_bspline() {
        let mut contour = Contour::new(true);
        contour.push(Segment::MoveTo { x: 0.0, y: 0.0 });
        contour.push(Segment::LineTo { x: 100.0, y: 0.0 });

        let curve = contour_to_bspline(&contour).unwrap();

        assert_eq!(curve.degree, 3);
        assert!(curve.weights.is_none());
        assert!(curve.control_points.len() >= 2);
    }

    #[test]
    fn test_quadratic_elevation() {
        let mut contour = Contour::new(true);
        contour.push(Segment::MoveTo { x: 0.0, y: 0.0 });
        contour.push(Segment::QuadTo {
            cx: 50.0,
            cy: 100.0,
            x: 100.0,
            y: 0.0,
        });

        let curve = contour_to_bspline(&contour).unwrap();

        assert_eq!(curve.degree, 3);
        assert!(curve.weights.is_none());
        // Should have start + 3 control points from quad
        assert!(curve.control_points.len() >= 4);
    }
}
