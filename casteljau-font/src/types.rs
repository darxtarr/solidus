use serde::{Deserialize, Serialize};
use solidus_geom::Contour;

/// Font metadata (casteljau output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMeta {
    /// Path to original font file (relative to this JSON)
    pub font_file: String,

    /// PostScript name
    pub postscript_name: String,

    /// Font family name
    pub family_name: String,

    /// Font style (Regular, Bold, Italic, etc.)
    pub style: String,

    /// Units per em (typically 1000 or 2048)
    pub units_per_em: u16,

    /// Variable font instance (if applicable)
    /// "default" = using default instance, null = static font
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable_instance: Option<String>,

    /// Glyph entries
    pub glyphs: Vec<GlyphEntry>,
}

/// Entry in font metadata pointing to a glyph file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphEntry {
    /// Internal glyph ID from font
    pub glyph_id: u16,

    /// Glyph name (if available)
    pub glyph_name: String,

    /// Unicode codepoints mapping to this glyph
    pub codepoints: Vec<String>,

    /// Path to glyph JSON file (relative to font directory)
    pub path: String,
}

/// Complete glyph data with primitives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphData {
    /// Font name
    pub font: String,

    /// Glyph name
    pub glyph_name: String,

    /// Glyph ID
    pub glyph_id: u16,

    /// Unicode codepoints
    pub codepoints: Vec<String>,

    /// Font units per em
    pub units_per_em: u16,

    /// Advance width (horizontal)
    pub advance_width: u16,

    /// Contours forming this glyph
    pub contours: Vec<Contour>,
}

impl GlyphData {
    /// Create glyph filename from codepoints and name
    ///
    /// Format: U+XXXX_name.json or glyphNNN_name.json
    pub fn filename(codepoints: &[u32], name: &str, glyph_id: u16) -> String {
        if let Some(&cp) = codepoints.first() {
            format!("U+{:04X}_{}.json", cp, sanitize_filename(name))
        } else {
            format!("glyph{}_{}.json", glyph_id, sanitize_filename(name))
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
