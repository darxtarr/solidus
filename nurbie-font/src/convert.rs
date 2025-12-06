use std::fs;
use std::path::Path;
use crate::bspline::contour_to_bspline;
use crate::types::{NurbieFontMeta, NurbieGlyphEntry, NurbieGlyphData};
use casteljau_font::{FontMeta, GlyphData};

const CHI_GEOMETRY_VERSION: u32 = 1;

/// Convert Casteljau primitives to Nurbie B-splines
pub fn convert_font(
    casteljau_dir: &Path,
    out_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read Casteljau font metadata
    let meta_path = casteljau_dir.join("font_meta.json");
    let meta_json = fs::read_to_string(&meta_path)?;
    let casteljau_meta: FontMeta = serde_json::from_str(&meta_json)?;

    // Create output directory
    fs::create_dir_all(out_dir)?;
    let glyphs_dir = out_dir.join("glyphs");
    fs::create_dir_all(&glyphs_dir)?;

    // Convert each glyph
    let mut nurbie_entries = Vec::new();

    for glyph_entry in &casteljau_meta.glyphs {
        // Read Casteljau glyph file
        let glyph_path = casteljau_dir.join(&glyph_entry.path);
        let glyph_json = fs::read_to_string(&glyph_path)?;
        let glyph_data: GlyphData = serde_json::from_str(&glyph_json)?;

        // Convert contours to B-splines
        let mut bspline_contours = Vec::new();
        for contour in &glyph_data.contours {
            let bspline = contour_to_bspline(contour)?;
            bspline_contours.push(bspline);
        }

        // Parse codepoints to u32 for filename generation
        let codepoints_u32: Vec<u32> = glyph_data
            .codepoints
            .iter()
            .filter_map(|s| {
                if s.starts_with("U+") {
                    u32::from_str_radix(&s[2..], 16).ok()
                } else {
                    None
                }
            })
            .collect();

        // Create nurbie filename
        let nurbie_filename = NurbieGlyphData::filename(
            &codepoints_u32,
            &glyph_data.glyph_name,
            glyph_data.glyph_id,
        );

        // Create nurbie glyph data
        let nurbie_glyph = NurbieGlyphData {
            font: glyph_data.font.clone(),
            glyph_name: glyph_data.glyph_name.clone(),
            glyph_id: glyph_data.glyph_id,
            codepoints: glyph_data.codepoints.clone(),
            chi_geometry_version: CHI_GEOMETRY_VERSION,
            source_casteljau: format!("../casteljau/{}/{}",
                casteljau_dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown"),
                glyph_entry.path
            ),
            units_per_em: glyph_data.units_per_em,
            advance_width: glyph_data.advance_width,
            contours: bspline_contours,
        };

        // Write nurbie glyph file
        let nurbie_path = glyphs_dir.join(&nurbie_filename);
        let nurbie_json = serde_json::to_string_pretty(&nurbie_glyph)?;
        fs::write(&nurbie_path, nurbie_json)?;

        // Add to metadata
        nurbie_entries.push(NurbieGlyphEntry {
            glyph_id: glyph_data.glyph_id,
            glyph_name: glyph_data.glyph_name.clone(),
            codepoints: glyph_data.codepoints.clone(),
            path: format!("glyphs/{}", nurbie_filename),
        });
    }

    // Write nurbie font metadata
    let nurbie_meta = NurbieFontMeta {
        source_casteljau_font_meta: format!(
            "../casteljau/{}/font_meta.json",
            casteljau_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
        ),
        chi_geometry_version: CHI_GEOMETRY_VERSION,
        glyphs: nurbie_entries,
    };

    let meta_path = out_dir.join("font_meta.json");
    let meta_json = serde_json::to_string_pretty(&nurbie_meta)?;
    fs::write(meta_path, meta_json)?;

    Ok(())
}
