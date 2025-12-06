use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Round-trip test: Extract Cantarell → Convert to B-splines → Verify properties
#[test]
fn test_cantarell_roundtrip() {
    // Find Cantarell font (system font)
    let cantarell_path = "/usr/share/fonts/abattis-cantarell-vf-fonts/Cantarell-VF.otf";

    if !PathBuf::from(cantarell_path).exists() {
        eprintln!("Skipping test: Cantarell font not found at {}", cantarell_path);
        return;
    }

    // Create temp directories
    let temp = TempDir::new().expect("Failed to create temp dir");
    let casteljau_dir = temp.path().join("casteljau");
    let nurbie_dir = temp.path().join("nurbie");

    // Extract with casteljau
    casteljau_font::extract_font(
        &PathBuf::from(cantarell_path),
        &casteljau_dir,
    )
    .expect("Failed to extract font");

    // Convert with nurbie
    nurbie_font::convert_font(&casteljau_dir, &nurbie_dir)
        .expect("Failed to convert to B-splines");

    // Read nurbie font metadata
    let nurbie_meta_path = nurbie_dir.join("font_meta.json");
    let nurbie_meta_json = fs::read_to_string(nurbie_meta_path)
        .expect("Failed to read nurbie font_meta.json");
    let nurbie_meta: nurbie_font::NurbieFontMeta =
        serde_json::from_str(&nurbie_meta_json).expect("Failed to parse nurbie metadata");

    assert_eq!(nurbie_meta.chi_geometry_version, 1, "Expected chi_geometry_version=1");
    assert!(!nurbie_meta.glyphs.is_empty(), "Expected at least one glyph");

    // Verify specific glyphs: A, S, space
    let test_glyphs = vec![
        ("U+0041", "A"),
        ("U+0053", "S"),
        ("U+0020", "space"),
    ];

    for (codepoint, name) in test_glyphs {
        let glyph_entry = nurbie_meta
            .glyphs
            .iter()
            .find(|g| g.codepoints.contains(&codepoint.to_string()));

        if let Some(entry) = glyph_entry {
            assert_eq!(entry.glyph_name, name, "Expected glyph name {}", name);

            // Read glyph file
            let glyph_path = nurbie_dir.join(&entry.path);
            let glyph_json = fs::read_to_string(&glyph_path)
                .expect(&format!("Failed to read glyph file for {}", name));
            let glyph_data: nurbie_font::NurbieGlyphData =
                serde_json::from_str(&glyph_json)
                    .expect(&format!("Failed to parse glyph data for {}", name));

            // Verify traceability
            assert_eq!(glyph_data.chi_geometry_version, 1);
            assert!(glyph_data.source_casteljau.contains("casteljau"));

            // Verify contours
            for (i, contour) in glyph_data.contours.iter().enumerate() {
                // All contours must be degree 3
                assert_eq!(
                    contour.degree, 3,
                    "Glyph {} contour {} expected degree 3, got {}",
                    name, i, contour.degree
                );

                // All contours must be non-rational
                assert!(
                    contour.weights.is_none(),
                    "Glyph {} contour {} should have weights=null",
                    name, i
                );

                // Verify knot vector properties
                assert!(
                    !contour.knots.is_empty(),
                    "Glyph {} contour {} has empty knot vector",
                    name, i
                );

                // Check clamped: first 4 knots should be 0.0
                for j in 0..4 {
                    assert_eq!(
                        contour.knots[j], 0.0,
                        "Glyph {} contour {} knot[{}] should be 0.0 (clamped start)",
                        name, i, j
                    );
                }

                // Check clamped: last 4 knots should be 1.0
                let len = contour.knots.len();
                for j in 0..4 {
                    assert_eq!(
                        contour.knots[len - 4 + j], 1.0,
                        "Glyph {} contour {} knot[{}] should be 1.0 (clamped end)",
                        name, i, len - 4 + j
                    );
                }

                // Verify knots are non-decreasing
                for j in 1..contour.knots.len() {
                    assert!(
                        contour.knots[j] >= contour.knots[j - 1],
                        "Glyph {} contour {} knots not non-decreasing at index {}",
                        name, i, j
                    );
                }

                // Verify knot count matches control points
                let n = contour.control_points.len() - 1;
                let expected_knots = n + contour.degree + 2;
                assert_eq!(
                    contour.knots.len(),
                    expected_knots,
                    "Glyph {} contour {} has {} knots, expected {} for {} control points and degree {}",
                    name, i, contour.knots.len(), expected_knots, contour.control_points.len(), contour.degree
                );

                // approx_error should be null for exact elevation
                assert!(
                    contour.approx_error.is_none(),
                    "Glyph {} contour {} approx_error should be null for exact elevation",
                    name, i
                );
            }

            println!("✓ Glyph {} verified: {} contours, all degree 3, properly clamped", name, glyph_data.contours.len());
        } else {
            println!("⚠ Glyph {} ({}) not found in output", name, codepoint);
        }
    }
}

/// Test that casteljau extraction preserves font metadata
#[test]
fn test_casteljau_metadata() {
    let cantarell_path = "/usr/share/fonts/abattis-cantarell-vf-fonts/Cantarell-VF.otf";

    if !PathBuf::from(cantarell_path).exists() {
        eprintln!("Skipping test: Cantarell font not found");
        return;
    }

    let temp = TempDir::new().expect("Failed to create temp dir");
    let casteljau_dir = temp.path().join("casteljau");

    casteljau_font::extract_font(&PathBuf::from(cantarell_path), &casteljau_dir)
        .expect("Failed to extract font");

    let meta_path = casteljau_dir.join("font_meta.json");
    let meta_json = fs::read_to_string(meta_path).expect("Failed to read font_meta.json");
    let meta: casteljau_font::FontMeta =
        serde_json::from_str(&meta_json).expect("Failed to parse metadata");

    assert_eq!(meta.family_name, "Cantarell");
    assert_eq!(meta.units_per_em, 1000);
    assert!(meta.variable_instance.is_some(), "Cantarell is a variable font");
    assert_eq!(meta.variable_instance.unwrap(), "default");
    assert!(!meta.glyphs.is_empty());

    println!("✓ Cantarell metadata verified: {} glyphs, variable font (default instance)", meta.glyphs.len());
}
