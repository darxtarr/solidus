// Nurbie-font library: primitive segments → cubic B-splines
// Curve conversion logic, shared between binary and tests

pub mod bspline;
pub mod convert;
pub mod types;

pub use convert::convert_font;
pub use types::*;
