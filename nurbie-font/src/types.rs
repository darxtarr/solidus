use serde::{Deserialize, Serialize};
use solidus_geom::BSplineCurve;

/// Font metadata (nurbie output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurbieFontMeta {
    /// Path to source Casteljau font_meta.json
    pub source_casteljau_font_meta: String,

    /// Chi geometry version
    pub chi_geometry_version: u32,

    /// Glyph entries
    pub glyphs: Vec<NurbieGlyphEntry>,
}

/// Entry in nurbie metadata pointing to a glyph NURBS file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurbieGlyphEntry {
    /// Glyph ID
    pub glyph_id: u16,

    /// Glyph name
    pub glyph_name: String,

    /// Unicode codepoints
    pub codepoints: Vec<String>,

    /// Path to NURBS file (relative to font directory)
    pub path: String,
}

/// Complete glyph NURBS data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurbieGlyphData {
    /// Font name
    pub font: String,

    /// Glyph name
    pub glyph_name: String,

    /// Glyph ID
    pub glyph_id: u16,

    /// Unicode codepoints
    pub codepoints: Vec<String>,

    /// Chi geometry version
    pub chi_geometry_version: u32,

    /// Path to source Casteljau glyph file
    pub source_casteljau: String,

    /// Font units per em
    pub units_per_em: u16,

    /// Advance width
    pub advance_width: u16,

    /// B-spline curves for each contour
    pub contours: Vec<BSplineCurve>,
}

impl NurbieGlyphData {
    /// Create nurbie filename from codepoints and name
    ///
    /// Format: U+XXXX_name.nurbs.json or glyphNNN_name.nurbs.json
    pub fn filename(codepoints: &[u32], name: &str, glyph_id: u16) -> String {
        if let Some(&cp) = codepoints.first() {
            format!("U+{:04X}_{}.nurbs.json", cp, sanitize_filename(name))
        } else {
            format!("glyph{}_{}.nurbs.json", glyph_id, sanitize_filename(name))
        }
    }
}

/// Sanitize a string for use in filename
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
