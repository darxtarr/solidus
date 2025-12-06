use solidus_geom::{Contour, Segment};
use nurbie_font::bspline::contour_to_bspline;

#[test]
fn test_synthetic_triangle() {
    // Create a simple triangular contour
    let mut contour = Contour::new(true);
    contour.push(Segment::MoveTo { x: 0.0, y: 0.0 });
    contour.push(Segment::LineTo { x: 100.0, y: 0.0 });
    contour.push(Segment::LineTo { x: 50.0, y: 100.0 });
    contour.push(Segment::LineTo { x: 0.0, y: 0.0 });

    // Convert to B-spline
    let bspline = contour_to_bspline(&contour).expect("Failed to convert triangle");

    // Verify properties
    assert_eq!(bspline.degree, 3, "Expected cubic B-spline");
    assert!(bspline.weights.is_none(), "Expected non-rational");
    assert!(bspline.control_points.len() >= 4, "Expected at least 4 control points");

    // Verify knot vector
    assert!(!bspline.knots.is_empty(), "Knot vector should not be empty");
    assert_eq!(*bspline.knots.first().unwrap(), 0.0, "First knot should be 0");
    assert_eq!(*bspline.knots.last().unwrap(), 1.0, "Last knot should be 1");

    // Verify knots are non-decreasing
    for i in 1..bspline.knots.len() {
        assert!(
            bspline.knots[i] >= bspline.knots[i - 1],
            "Knots must be non-decreasing"
        );
    }
}

#[test]
fn test_synthetic_quadratic_curve() {
    // Create a simple quadratic Bézier curve
    let mut contour = Contour::new(false);
    contour.push(Segment::MoveTo { x: 0.0, y: 0.0 });
    contour.push(Segment::QuadTo {
        cx: 50.0,
        cy: 100.0,
        x: 100.0,
        y: 0.0,
    });

    // Convert to B-spline (should elevate to cubic)
    let bspline = contour_to_bspline(&contour).expect("Failed to convert quadratic");

    // Verify it's cubic
    assert_eq!(bspline.degree, 3);
    assert!(bspline.weights.is_none());

    // Should have start point + 3 control points from elevated quadratic
    assert!(bspline.control_points.len() >= 4);

    // Validate the curve
    bspline.validate().expect("B-spline should be valid");
}

#[test]
fn test_synthetic_cubic_curve() {
    // Create a cubic Bézier curve
    let mut contour = Contour::new(false);
    contour.push(Segment::MoveTo { x: 0.0, y: 0.0 });
    contour.push(Segment::CubicTo {
        cx1: 33.0,
        cy1: 100.0,
        cx2: 67.0,
        cy2: 100.0,
        x: 100.0,
        y: 0.0,
    });

    // Convert to B-spline (should stay cubic)
    let bspline = contour_to_bspline(&contour).expect("Failed to convert cubic");

    // Verify it's cubic
    assert_eq!(bspline.degree, 3);
    assert!(bspline.weights.is_none());

    // Should have control points
    assert!(bspline.control_points.len() >= 4);

    // Validate the curve
    bspline.validate().expect("B-spline should be valid");
}

#[test]
fn test_mixed_segment_types() {
    // Create a contour with mixed segment types
    let mut contour = Contour::new(true);
    contour.push(Segment::MoveTo { x: 0.0, y: 0.0 });
    contour.push(Segment::LineTo { x: 50.0, y: 0.0 });
    contour.push(Segment::QuadTo {
        cx: 75.0,
        cy: 25.0,
        x: 100.0,
        y: 50.0,
    });
    contour.push(Segment::CubicTo {
        cx1: 100.0,
        cy1: 75.0,
        cx2: 75.0,
        cy2: 100.0,
        x: 50.0,
        y: 100.0,
    });
    contour.push(Segment::LineTo { x: 0.0, y: 100.0 });
    contour.push(Segment::LineTo { x: 0.0, y: 0.0 });

    // Convert to B-spline
    let bspline = contour_to_bspline(&contour).expect("Failed to convert mixed contour");

    // Verify properties
    assert_eq!(bspline.degree, 3);
    assert!(bspline.weights.is_none());

    // Validate the curve
    bspline.validate().expect("B-spline should be valid");
}
