# Complete Solidus Output Format (v1)

## Format Contracts

These are **invariants**, not implementation details:

### Coordinate System Contract
- **Font space preserved**: All coordinates remain in font units (typically 1000 or 2048 units per em)
- **Y-axis orientation**: Y-up (font designer's coordinate system)
- **No normalization**: No flipping, scaling, or transformation to world/screen space
- **Downstream transformation**: Conversion to Chi's world coordinates happens in the marking/layout layer

### B-spline Properties Contract (v1)
- **Degree**: Always 3 (cubic)
- **Rational**: Always non-rational (weights = null)
- **Knot vectors**: Clamped (start/end repeated degree+1 times: [0,0,0,0, ..., 1,1,1,1])
- **Parameterization**: Chord-length (non-uniform spacing based on control point distances)
- **Knot count**: `num_knots = num_control_points + degree + 1`
- **Interior knots**: Non-decreasing sequence

### Traceability Contract
- **Nurbie font_meta.json** MUST include:
  - `source_casteljau_font_meta`: Relative path to Casteljau font_meta.json
  - `chi_geometry_version`: Integer version (currently 1)
- **Nurbie glyph files** MUST include:
  - `source_casteljau`: Relative path to source Casteljau glyph file
  - `chi_geometry_version`: Integer version (currently 1)
  - `approx_error`: Always present (null for exact, object for fitted)

### Version Evolution Contract
- **chi_geometry_version** changes when:
  - Knot parameterization changes (chord-length → centripetal)
  - Degree changes (cubic → higher order)
  - Rational vs non-rational changes
- **chi_geometry_version** does NOT change when:
  - Curve fitting is applied (approx_error populated instead)
  - Output format adds optional fields

---

# Complete Solidus Output Format (v1)

## Casteljau font_meta.json

```json
{
  "font_file": "../../fonts_in/Cantarell-VF.otf",
  "postscript_name": "Cantarell-Regular",
  "family_name": "Cantarell",
  "style": "Regular",
  "units_per_em": 1000,
  "variable_instance": "default",
  "glyphs": [
    {
      "glyph_id": 1,
      "glyph_name": "A",
      "codepoints": ["U+0041"],
      "path": "glyphs/U+0041_A.json"
    }
  ]
}
```

**Key fields:**
- `variable_instance`: `"default"` for variable fonts, omitted for static fonts
- Provides traceability back to original font file

## Nurbie font_meta.json

```json
{
  "source_casteljau_font_meta": "../casteljau/Cantarell-VF/font_meta.json",
  "chi_geometry_version": 1,
  "glyphs": [
    {
      "glyph_id": 1,
      "glyph_name": "A",
      "codepoints": ["U+0041"],
      "path": "glyphs/U+0041_A.nurbs.json"
    }
  ]
}
```

**Key fields:**
- `source_casteljau_font_meta`: Link back to Casteljau extraction
- `chi_geometry_version`: Version knob (currently 1)

## Nurbie glyph file (complete)

```json
{
  "font": "Cantarell-Regular",
  "glyph_name": "A",
  "glyph_id": 1,
  "codepoints": ["U+0041"],
  "chi_geometry_version": 1,
  "source_casteljau": "../casteljau/Cantarell-VF/glyphs/U+0041_A.json",
  "units_per_em": 1000,
  "advance_width": 626,
  "contours": [
    {
      "closed": true,
      "degree": 3,
      "knots": [0.0, 0.0, 0.0, 0.0, 0.05, 0.12, ..., 1.0, 1.0, 1.0, 1.0],
      "control_points": [
        {"x": 7.0, "y": 0.0},
        {"x": 36.0, "y": 0.0},
        ...
      ],
      "weights": null,
      "approx_error": null
    }
  ]
}
```

**Key fields:**
- `chi_geometry_version`: Version at glyph level (regeneration tracking)
- `source_casteljau`: Link to source primitive file
- `approx_error`: Always present, `null` for exact (future: populated for fitted curves)
- `weights`: `null` for non-rational (v1 always non-rational)

## Traceability Chain

```
Original Font (Cantarell-VF.otf)
    ↓
Casteljau font_meta.json
    → variable_instance: "default"
    → glyphs[].path
        ↓
Casteljau glyph files (primitives)
    → explicit segments (MoveTo, LineTo, CubicTo)
        ↓
Nurbie font_meta.json
    → source_casteljau_font_meta: "../casteljau/..."
    → chi_geometry_version: 1
        ↓
Nurbie glyph files (B-splines)
    → source_casteljau: "../casteljau/.../U+0041_A.json"
    → chi_geometry_version: 1
    → approx_error: null (exact) or {max, mean, metric}
```

## Version Evolution Scenarios

### Scenario 1: Change parameterization (uniform → centripetal)
- Bump `chi_geometry_version` to 2
- Regenerate all fonts with new version
- Old files remain at version 1, distinguishable

### Scenario 2: Add curve fitting
- Keep `chi_geometry_version` at 1
- Populate `approx_error` with actual metrics
- Downstream tools can filter by error bounds

### Scenario 3: Support variable font axes
- Add `variable_instance: "wght=700,wdth=100"` to Casteljau
- Downstream knows this isn't default instance
- Can regenerate for different axis values

## Benefits

✅ **Regeneration**: Can always trace back to source and re-canonize
✅ **Versioning**: Clear knob to turn when algorithms change
✅ **Comparison**: Can compare v1 vs v2 outputs side-by-side
✅ **Quality control**: approx_error distinguishes exact vs fitted
✅ **Variable fonts**: Explicit marker prevents confusion about which instance
