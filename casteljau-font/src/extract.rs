use std::fs;
use std::path::Path;
use ttf_parser::{Face, GlyphId};
use solidus_geom::{Contour, Point2, Segment};
use crate::types::{FontMeta, GlyphEntry, GlyphData};

/// Extract font glyphs to primitive segments (ASCII printable range)
pub fn extract_font(font_path: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    extract_font_filtered(font_path, out_dir, None)
}

/// Extract font glyphs to primitive segments with optional glyph filter
///
/// If filter is None, extracts ASCII printable range (U+0020-U+007E).
/// If filter is Some, extracts only the specified codepoints.
pub fn extract_font_filtered(
    font_path: &Path,
    out_dir: &Path,
    glyph_filter: Option<&[u32]>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read font file
    let font_data = fs::read(font_path)?;
    let face = Face::parse(&font_data, 0)?;

    // Create output directory
    fs::create_dir_all(out_dir)?;
    let glyphs_dir = out_dir.join("glyphs");
    fs::create_dir_all(&glyphs_dir)?;

    // Extract font metadata
    let postscript_name = face
        .names()
        .into_iter()
        .find(|n| n.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
        .and_then(|n| n.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let family_name = face
        .names()
        .into_iter()
        .find(|n| n.name_id == ttf_parser::name_id::FAMILY)
        .and_then(|n| n.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let style = face
        .names()
        .into_iter()
        .find(|n| n.name_id == ttf_parser::name_id::SUBFAMILY)
        .and_then(|n| n.to_string())
        .unwrap_or_else(|| "Regular".to_string());

    let units_per_em = face.units_per_em();

    // Check if this is a variable font
    let variable_instance = if face.is_variable() {
        Some("default".to_string())
    } else {
        None
    };

    // Determine which glyphs to extract
    let codepoints_to_extract: Vec<u32> = match glyph_filter {
        Some(filter) => filter.to_vec(),
        None => (0x20..=0x7E).collect(), // ASCII printable + space
    };

    let mut glyph_entries = Vec::new();

    for codepoint in codepoints_to_extract {
        if let Some(glyph_id) = face.glyph_index(char::from_u32(codepoint).unwrap()) {
            let glyph_name = face
                .glyph_name(glyph_id)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("glyph{}", glyph_id.0));

            let codepoints_str = vec![format!("U+{:04X}", codepoint)];
            let filename = GlyphData::filename(&[codepoint], &glyph_name, glyph_id.0);

            // Extract glyph contours
            let contours = extract_glyph(&face, glyph_id)?;

            // Get advance width
            let advance_width = face.glyph_hor_advance(glyph_id).unwrap_or(0);

            // Create glyph data
            let glyph_data = GlyphData {
                font: postscript_name.clone(),
                glyph_name: glyph_name.clone(),
                glyph_id: glyph_id.0,
                codepoints: codepoints_str.clone(),
                units_per_em,
                advance_width,
                contours,
            };

            // Write glyph file
            let glyph_path = glyphs_dir.join(&filename);
            let glyph_json = serde_json::to_string_pretty(&glyph_data)?;
            fs::write(&glyph_path, glyph_json)?;

            // Add to metadata
            glyph_entries.push(GlyphEntry {
                glyph_id: glyph_id.0,
                glyph_name,
                codepoints: codepoints_str,
                path: format!("glyphs/{}", filename),
            });
        }
    }

    // Write font metadata
    let font_meta = FontMeta {
        font_file: font_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| format!("../../fonts_in/{}", s))
            .unwrap_or_else(|| "unknown.ttf".to_string()),
        postscript_name,
        family_name,
        style,
        units_per_em,
        variable_instance,
        glyphs: glyph_entries,
    };

    let meta_path = out_dir.join("font_meta.json");
    let meta_json = serde_json::to_string_pretty(&font_meta)?;
    fs::write(meta_path, meta_json)?;

    Ok(())
}

/// Extract contours from a single glyph
fn extract_glyph(face: &Face, glyph_id: GlyphId) -> Result<Vec<Contour>, Box<dyn std::error::Error>> {
    let mut contours = Vec::new();
    let mut current_contour: Option<Contour> = None;
    let mut current_point = Point2::new(0.0, 0.0);

    // For TrueType quadratic curves: track implied on-curve points
    let mut last_off_curve: Option<Point2> = None;

    let mut builder = OutlineBuilder {
        current_contour: &mut current_contour,
        current_point: &mut current_point,
        last_off_curve: &mut last_off_curve,
        contours: &mut contours,
    };

    face.outline_glyph(glyph_id, &mut builder);

    // Close final contour if needed
    if let Some(mut contour) = current_contour.take() {
        contour.close_if_needed();
        contours.push(contour);
    }

    Ok(contours)
}

/// Outline builder for ttf-parser
struct OutlineBuilder<'a> {
    current_contour: &'a mut Option<Contour>,
    current_point: &'a mut Point2,
    last_off_curve: &'a mut Option<Point2>,
    contours: &'a mut Vec<Contour>,
}

impl ttf_parser::OutlineBuilder for OutlineBuilder<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        // Close previous contour
        if let Some(mut contour) = self.current_contour.take() {
            contour.close_if_needed();
            self.contours.push(contour);
        }

        // Start new contour
        let mut contour = Contour::new(true);
        contour.push(Segment::MoveTo {
            x: x as f64,
            y: y as f64,
        });

        *self.current_point = Point2::new(x as f64, y as f64);
        *self.last_off_curve = None;
        *self.current_contour = Some(contour);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        if let Some(ref mut contour) = self.current_contour {
            contour.push(Segment::LineTo {
                x: x as f64,
                y: y as f64,
            });
            *self.current_point = Point2::new(x as f64, y as f64);
            *self.last_off_curve = None;
        }
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        if let Some(ref mut contour) = self.current_contour {
            // Handle implied on-curve points in TrueType
            // If we have a previous off-curve point, insert implied on-curve
            if let Some(prev_off) = self.last_off_curve.take() {
                let implied = Point2::new(
                    (prev_off.x + cx as f64) / 2.0,
                    (prev_off.y + cy as f64) / 2.0,
                );

                // Emit quadratic to implied point
                contour.push(Segment::QuadTo {
                    cx: prev_off.x,
                    cy: prev_off.y,
                    x: implied.x,
                    y: implied.y,
                });
                *self.current_point = implied;
            }

            // Check if endpoint is on-curve or off-curve
            // ttf-parser always gives us the actual endpoint in quad_to
            contour.push(Segment::QuadTo {
                cx: cx as f64,
                cy: cy as f64,
                x: x as f64,
                y: y as f64,
            });

            *self.current_point = Point2::new(x as f64, y as f64);
            *self.last_off_curve = None;
        }
    }

    fn curve_to(&mut self, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32) {
        if let Some(ref mut contour) = self.current_contour {
            contour.push(Segment::CubicTo {
                cx1: cx1 as f64,
                cy1: cy1 as f64,
                cx2: cx2 as f64,
                cy2: cy2 as f64,
                x: x as f64,
                y: y as f64,
            });
            *self.current_point = Point2::new(x as f64, y as f64);
            *self.last_off_curve = None;
        }
    }

    fn close(&mut self) {
        // TTF close means return to start point
        // Contour::close_if_needed will add LineTo if needed
    }
}
