use serde::{Deserialize, Serialize};
use crate::Point2;

/// Knot vector for B-splines
///
/// A non-decreasing sequence of parameter values.
/// For cubic curves (degree 3), need at least (n + 4) knots for n control points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnotVector {
    pub knots: Vec<f64>,
}

impl KnotVector {
    /// Create a clamped uniform knot vector
    ///
    /// For degree d and n control points:
    /// - First (d+1) knots are 0.0
    /// - Last (d+1) knots are 1.0
    /// - Interior knots uniformly spaced
    pub fn clamped_uniform(degree: usize, num_control_points: usize) -> Self {
        let n = num_control_points - 1;
        let m = n + degree + 1;
        let mut knots = Vec::with_capacity(m + 1);

        // First (degree + 1) knots are 0
        for _ in 0..=degree {
            knots.push(0.0);
        }

        // Interior knots uniformly spaced
        let num_interior = m - 2 * degree - 1;
        if num_interior > 0 {
            for i in 1..=num_interior {
                let t = i as f64 / (num_interior + 1) as f64;
                knots.push(t);
            }
        }

        // Last (degree + 1) knots are 1
        for _ in 0..=degree {
            knots.push(1.0);
        }

        Self { knots }
    }

    /// Create a clamped chord-length knot vector
    ///
    /// Knots spaced proportional to distances between control points.
    /// Better for varying segment lengths.
    pub fn clamped_chord_length(degree: usize, control_points: &[Point2]) -> Self {
        let n = control_points.len() - 1;
        let m = n + degree + 1;
        let mut knots = Vec::with_capacity(m + 1);

        // First (degree + 1) knots are 0
        for _ in 0..=degree {
            knots.push(0.0);
        }

        // Compute chord lengths
        let mut chord_lengths = Vec::with_capacity(n);
        let mut total_length = 0.0;
        for i in 0..n {
            let length = control_points[i].distance_to(&control_points[i + 1]);
            chord_lengths.push(length);
            total_length += length;
        }

        // Interior knots based on accumulated chord length
        let num_interior = m - 2 * degree - 1;
        if num_interior > 0 {
            let segment_length = total_length / (num_interior + 1) as f64;

            for i in 1..=num_interior {
                let target = i as f64 * segment_length;

                // Find where this falls in chord lengths
                let mut sum = 0.0;
                for &len in &chord_lengths {
                    sum += len;
                    if sum >= target {
                        break;
                    }
                }

                let t = if total_length > 0.0 {
                    sum / total_length
                } else {
                    i as f64 / (num_interior + 1) as f64
                };

                knots.push(t.clamp(0.0, 1.0));
            }
        }

        // Last (degree + 1) knots are 1
        for _ in 0..=degree {
            knots.push(1.0);
        }

        Self { knots }
    }

    /// Validate knot vector properties
    pub fn validate(&self) -> Result<(), String> {
        if self.knots.is_empty() {
            return Err("Knot vector is empty".to_string());
        }

        // Check non-decreasing
        for i in 1..self.knots.len() {
            if self.knots[i] < self.knots[i - 1] {
                return Err(format!(
                    "Knots not non-decreasing at index {}: {} > {}",
                    i, self.knots[i - 1], self.knots[i]
                ));
            }
        }

        Ok(())
    }
}

/// Cubic B-spline curve (degree 3, non-rational)
///
/// Represents a smooth curve using control points and a knot vector.
/// All curves in Chi font pipeline are cubic, non-rational B-splines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BSplineCurve {
    /// Whether this curve is closed
    pub closed: bool,

    /// Degree of the curve (always 3 for cubic)
    pub degree: usize,

    /// Knot vector (non-decreasing parameter values)
    pub knots: Vec<f64>,

    /// Control points
    pub control_points: Vec<Point2>,

    /// Weights (None for non-rational, Some for rational)
    /// Chi pipeline uses non-rational, so this is always None for v0
    pub weights: Option<Vec<f64>>,

    /// Approximation error metrics (optional, for nurbie output)
    /// Always serialized (null for exact, populated for fitted curves)
    pub approx_error: Option<ApproxError>,
}

/// Approximation error metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproxError {
    /// Maximum distance from original curve (font units)
    pub max_distance: f64,

    /// Mean distance from original curve (font units)
    pub mean_distance: f64,

    /// What metric was used
    pub metric: String,
}

impl BSplineCurve {
    /// Create a new cubic B-spline curve
    pub fn new_cubic(control_points: Vec<Point2>, knots: KnotVector, closed: bool) -> Self {
        Self {
            closed,
            degree: 3,
            knots: knots.knots,
            control_points,
            weights: None,
            approx_error: None,
        }
    }

    /// Validate curve properties
    pub fn validate(&self) -> Result<(), String> {
        if self.degree != 3 {
            return Err(format!("Expected degree 3, got {}", self.degree));
        }

        if self.control_points.is_empty() {
            return Err("No control points".to_string());
        }

        let n = self.control_points.len() - 1;
        let expected_knots = n + self.degree + 2;

        if self.knots.len() != expected_knots {
            return Err(format!(
                "Expected {} knots for {} control points and degree {}, got {}",
                expected_knots,
                self.control_points.len(),
                self.degree,
                self.knots.len()
            ));
        }

        // Validate knot vector
        KnotVector { knots: self.knots.clone() }.validate()?;

        // Validate weights if present
        if let Some(ref weights) = self.weights {
            if weights.len() != self.control_points.len() {
                return Err(format!(
                    "Weights count {} doesn't match control points count {}",
                    weights.len(),
                    self.control_points.len()
                ));
            }
        }

        Ok(())
    }
}
