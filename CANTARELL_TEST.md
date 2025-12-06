# Cantarell Font Pipeline Test Results

## Test Font
- **Font**: Cantarell Variable Font (Cantarell-VF.otf)
- **Type**: OpenType/CFF (cubic Bézier curves)
- **Designer**: Dave Crossland (GNOME humanist sans-serif)
- **Units per em**: 1000
- **Style**: Regular

## Extraction Results

### Casteljau (TTF → Primitives)
- ✅ Successfully extracted 95 glyphs (ASCII printable range)
- ✅ Output format: JSON with explicit segments
- ✅ Segment types used: MoveTo, LineTo, CubicTo
- ✅ No implied points (all explicit)
- ✅ Coordinate system: Y-up, font units preserved

### Nurbie (Primitives → B-splines)
- ✅ Successfully converted all 95 glyphs to cubic B-splines
- ✅ All non-empty glyphs: degree=3, weights=null
- ✅ Knot vectors: Clamped (0.0⁴ ... 1.0⁴)
- ✅ Parameterization: Chord-length (non-uniform spacing)
- ✅ Empty glyphs preserved (e.g., space character)

## Sample Glyphs Inspected

### Letter 'A' (U+0041)
- **Contours**: 2 (outer triangle + crossbar)
- **Segment types**: MoveTo, LineTo only
- **Primitive segments**: 9 lines across 2 contours
- **B-spline**: 25 control points, 29 knots (first contour)

### Letter 'S' (U+0053)
- **Contours**: 2 (outer + inner curves)
- **Segment types**: MoveTo, LineTo, CubicTo
- **Primitive segments**: Mix of cubic curves and lines
- **B-spline**: 37 control points, 41 knots (first contour)
- **Knot spacing**: Non-uniform chord-length (verified)

## Validation

✅ All 95 glyphs extracted successfully
✅ All 95 glyphs converted to B-splines
✅ All non-empty glyphs have degree=3
✅ All glyphs have weights=null (non-rational)
✅ Knot vectors properly clamped (start/end repeated 4×)
✅ Interior knots are non-decreasing
✅ Knot spacing follows chord-length parameterization
✅ Empty glyphs (space) handled correctly

## Commands Used

```bash
# Extract primitives
./target/release/casteljau-font \
  --font chi_fonts/fonts_in/Cantarell-VF.otf \
  --out-dir chi_fonts/casteljau/Cantarell-VF

# Convert to B-splines
./target/release/nurbie-font \
  --casteljau-dir chi_fonts/casteljau/Cantarell-VF \
  --out-dir chi_fonts/nurbie/Cantarell-VF
```

## File Structure

```
chi_fonts/
├── fonts_in/
│   └── Cantarell-VF.otf
├── casteljau/
│   └── Cantarell-VF/
│       ├── font_meta.json
│       └── glyphs/
│           ├── U+0041_A.json
│           ├── U+0053_S.json
│           └── ... (95 total)
└── nurbie/
    └── Cantarell-VF/
        ├── font_meta.json
        └── glyphs/
            ├── U+0041_A.nurbs.json
            ├── U+0053_S.nurbs.json
            └── ... (95 total)
```

## Example Output: Letter 'A' Casteljau

```json
{
  "font": "Cantarell-Regular",
  "glyph_name": "A",
  "glyph_id": 1,
  "codepoints": ["U+0041"],
  "units_per_em": 1000,
  "advance_width": 626,
  "contours": [
    {
      "closed": true,
      "segments": [
        { "type": "MoveTo", "x": 7.0, "y": 0.0 },
        { "type": "LineTo", "x": 94.0, "y": 0.0 },
        { "type": "LineTo", "x": 321.0, "y": 636.0 },
        ...
      ]
    },
    ...
  ]
}
```

## Example Output: Letter 'A' Nurbie

```json
{
  "font": "Cantarell-Regular",
  "glyph_name": "A",
  "contours": [
    {
      "closed": true,
      "degree": 3,
      "knots": [0.0, 0.0, 0.0, 0.0, ..., 1.0, 1.0, 1.0, 1.0],
      "control_points": [
        { "x": 7.0, "y": 0.0 },
        { "x": 36.0, "y": 0.0 },
        ...
      ],
      "weights": null
    }
  ]
}
```

## Conclusion

🎉 **Pipeline fully functional with real-world font!**

The Solidus font geometry pipeline successfully processes a production-quality variable font (Cantarell), preserving all geometric information while transforming from TTF/OTF primitives to canonical cubic B-splines suitable for Chi's spatial computing substrate.
