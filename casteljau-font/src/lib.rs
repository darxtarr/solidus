// Casteljau-font library: TTF/OTF → primitive glyph segments
// Font extraction logic, shared between binary and tests

pub mod extract;
pub mod types;

pub use extract::{extract_font, extract_font_filtered};
pub use types::*;
