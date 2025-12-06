use serde::{Deserialize, Serialize};
use crate::{Point2, Segment};

/// A contour: a sequence of segments forming a path
///
/// In font glyphs, contours are typically closed shapes.
/// Contours should start with MoveTo and contain explicit segments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contour {
    /// Whether this contour is closed (most glyph outlines are)
    pub closed: bool,

    /// Segments forming the path
    /// First segment should be MoveTo
    pub segments: Vec<Segment>,
}

impl Contour {
    pub fn new(closed: bool) -> Self {
        Self {
            closed,
            segments: Vec::new(),
        }
    }

    /// Add a segment to this contour
    pub fn push(&mut self, segment: Segment) {
        self.segments.push(segment);
    }

    /// Get the starting point (from first MoveTo)
    pub fn start_point(&self) -> Option<Point2> {
        self.segments.first().and_then(|s| match s {
            Segment::MoveTo { x, y } => Some(Point2::new(*x, *y)),
            _ => None,
        })
    }

    /// Get the current endpoint (last segment's endpoint)
    pub fn current_point(&self) -> Option<Point2> {
        self.segments.last().and_then(|s| s.endpoint())
    }

    /// Close the contour by adding explicit LineTo if needed
    pub fn close_if_needed(&mut self) {
        if !self.closed {
            return;
        }

        let start = match self.start_point() {
            Some(p) => p,
            None => return,
        };

        let current = match self.current_point() {
            Some(p) => p,
            None => return,
        };

        // If not already at start, add closing line
        const EPSILON: f64 = 1e-9;
        if (current.x - start.x).abs() > EPSILON || (current.y - start.y).abs() > EPSILON {
            self.segments.push(Segment::LineTo {
                x: start.x,
                y: start.y,
            });
        }
    }
}
