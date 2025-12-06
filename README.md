# Solidus Font Geometry Pipeline

A self-contained Rust pipeline for converting TrueType/OpenType fonts into canonical cubic B-splines for the Chi/Chorus spatial computing ecosystem.

## What Is Solidus?

Solidus is the curve and font geometry lab for the Chi / Chorus spatial computing system.

It turns real fonts (TTF/OTF) into canonical cubic B-splines in "Chi space," preserving full traceability back to the original outlines. These curves are then used by higher-level components to print and etch text onto 3D panels (solids) and to render floating labels (holos) in Chi's cockpit.

In v1 Solidus focuses on:
- **Exact extraction** of glyph outlines into explicit Bézier segments (Casteljau),
- **Conversion** to cubic B-splines with chord-length parameterization (Nurbie),
- A **tiny SVG debug tool** (Nurbie-mark) to visualize the curves.

Later, the same curve machinery will be re-used for Huion strokes, icon outlines, and other geometric elements in Chi.

## Overview

Solidus implements a two-stage font geometry pipeline:

1. **casteljau-font**: Extracts glyph outlines from TTF/OTF fonts into primitive segments (MoveTo, LineTo, QuadTo, CubicTo)
2. **nurbie-font**: Converts primitive segments into canonical cubic B-splines (Chi's curve dialect)

This is a "shed project" — clarity and correctness over performance. Minimal dependencies, inspectable outputs.

## Architecture

### Four Crates

- **solidus-geom**: Shared geometry primitives
  - `Point2`: 2D points in font space
  - `Segment`: Primitive curve segments
  - `Contour`: Paths made of segments
  - `BSplineCurve`: Cubic non-rational B-splines
  - `KnotVector`: Chord-length parameterization
  - De Boor evaluation algorithm

- **casteljau-font**: Font → primitive segments
  - Reads TTF/OTF using `ttf-parser`
  - Resolves implied TrueType points
  - Outputs explicit segments per glyph
  - JSON format: `font_meta.json` + `glyphs/*.json`

- **nurbie-font**: Primitive segments → B-splines
  - Exact degree elevation (quadratic → cubic)
  - Chord-length knot vectors
  - Non-rational curves (weights = null)
  - JSON format: `font_meta.json` + `glyphs/*.nurbs.json`

- **nurbie-mark**: B-splines → SVG visualization (debug tool)
  - Evaluates B-splines using de Boor's algorithm
  - Samples curves into polylines
  - Generates SVG with font metric guides (baseline, em-height, advance width)
  - Primarily for visual inspection and pipeline validation

### Coordinate System

All outputs preserve font space exactly:
- Y-up (font designer's coordinate system)
- Font units (typically 1000 or 2048 units per em)
- No flipping, no normalization

Transformation from font space → pane space → world 3D happens later in the marking/layout layer.

## Building

```bash
cargo build --release
```

Binaries will be in `target/release/`:
- `casteljau-font`
- `nurbie-font`
- `nurbie-mark`

## Usage

### Step 1: Extract Primitives

```bash
casteljau-font \
  --font /path/to/fonts_in/ChiSans-Regular.ttf \
  --out-dir chi_fonts/casteljau/ChiSans-Regular
```

Output structure:
```
chi_fonts/casteljau/ChiSans-Regular/
  font_meta.json          # Font metadata + glyph index
  glyphs/
    U+0041_A.json         # Glyph 'A' primitive segments
    U+0042_B.json
    ...
```

### Step 2: Convert to B-Splines

```bash
nurbie-font \
  --casteljau-dir chi_fonts/casteljau/ChiSans-Regular \
  --out-dir chi_fonts/nurbie/ChiSans-Regular
```

Output structure:
```
chi_fonts/nurbie/ChiSans-Regular/
  font_meta.json                # Nurbie metadata + glyph index
  glyphs/
    U+0041_A.nurbs.json         # Glyph 'A' cubic B-splines
    U+0042_B.nurbs.json
    ...
```

### Step 3: Visualize Glyphs (Optional)

```bash
nurbie-mark \
  --glyph chi_fonts/nurbie/ChiSans-Regular/glyphs/U+0041_A.nurbs.json \
  --output glyph_A.svg \
  --samples 50
```

This generates an SVG file with:
- The glyph outline sampled from B-splines
- Font metric guides (baseline, em-height, advance width)
- Useful for visual verification and debugging

See `CANTARELL_TEST.md` for examples of nurbie-mark output.

## File Formats

### Casteljau: font_meta.json

```json
{
  "font_file": "../../fonts_in/ChiSans-Regular.ttf",
  "postscript_name": "ChiSans-Regular",
  "family_name": "ChiSans",
  "style": "Regular",
  "units_per_em": 2048,
  "glyphs": [
    {
      "glyph_id": 36,
      "glyph_name": "A",
      "codepoints": ["U+0041"],
      "path": "glyphs/U+0041_A.json"
    }
  ]
}
```

### Casteljau: Glyph File

```json
{
  "font": "ChiSans-Regular",
  "glyph_name": "A",
  "glyph_id": 36,
  "codepoints": ["U+0041"],
  "units_per_em": 2048,
  "advance_width": 1216,
  "contours": [
    {
      "closed": true,
      "segments": [
        { "type": "MoveTo", "x": 100.0, "y": 0.0 },
        { "type": "LineTo", "x": 300.0, "y": 700.0 },
        { "type": "QuadTo", "cx": 400.0, "cy": 750.0, "x": 500.0, "y": 700.0 },
        { "type": "LineTo", "x": 100.0, "y": 0.0 }
      ]
    }
  ]
}
```

Segment types:
- `MoveTo { x, y }` - Move to point
- `LineTo { x, y }` - Straight line
- `QuadTo { cx, cy, x, y }` - Quadratic Bézier (one control point)
- `CubicTo { cx1, cy1, cx2, cy2, x, y }` - Cubic Bézier (two control points)

### Nurbie: font_meta.json

```json
{
  "source_casteljau_font_meta": "../casteljau/ChiSans-Regular/font_meta.json",
  "chi_geometry_version": 1,
  "glyphs": [
    {
      "glyph_id": 36,
      "glyph_name": "A",
      "codepoints": ["U+0041"],
      "path": "glyphs/U+0041_A.nurbs.json"
    }
  ]
}
```

### Nurbie: Glyph NURBS File

```json
{
  "font": "ChiSans-Regular",
  "glyph_name": "A",
  "glyph_id": 36,
  "codepoints": ["U+0041"],
  "chi_geometry_version": 1,
  "source_casteljau": "../casteljau/ChiSans-Regular/glyphs/U+0041_A.json",
  "units_per_em": 2048,
  "advance_width": 1216,
  "contours": [
    {
      "closed": true,
      "degree": 3,
      "knots": [0.0, 0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0, 1.0],
      "control_points": [
        { "x": 100.0, "y": 0.0 },
        { "x": 200.0, "y": 350.0 },
        { "x": 400.0, "y": 350.0 },
        { "x": 500.0, "y": 0.0 }
      ],
      "weights": null,
      "approx_error": null
    }
  ]
}
```

B-spline properties:
- `degree`: Always 3 (cubic)
- `knots`: Chord-length parameterized, clamped (start/end repeated degree+1 times)
- `control_points`: Cubic control points
- `weights`: Always null (non-rational)
- `approx_error`: Optional error metrics (not yet implemented)

## Testing

```bash
# Run all tests (unit + integration)
cargo test

# Run only integration tests
cargo test -p tests-integration

# Run specific test
cargo test test_synthetic_triangle
```

Integration tests verify:
- Triangle (straight lines)
- Quadratic curves (degree elevation)
- Cubic curves (identity transform)
- Mixed segment types

## Current Status (v0.1)

**Implemented:**
- ✅ TTF/OTF parsing (quadratic + cubic outlines)
- ✅ Explicit segment resolution (implied points)
- ✅ Exact degree elevation (quad → cubic)
- ✅ Chord-length knot parameterization
- ✅ JSON I/O for both stages
- ✅ ASCII printable glyph range (U+0020 - U+007E)
- ✅ Integration tests (synthetic geometry)

**Not Yet Implemented:**
- ❌ Full Unicode glyph extraction (currently ASCII only)
- ❌ Approximation error metrics
- ❌ Curve simplification/fitting
- ❌ Real font smoke tests (requires sample font in repo)

## Design Principles

1. **Lossless extraction**: Casteljau preserves exact font outlines
2. **Exact elevation**: Quadratic → cubic uses exact math (zero error)
3. **Inspectable outputs**: JSON for human verification
4. **Coordinate preservation**: Font space maintained throughout
5. **Reusable geometry**: `solidus-geom` designed for Huion strokes, cockpit curves

## Future Extensions

- **Curve fitting**: Reduce control point count while maintaining error bounds
- **Approximation metrics**: Compute max/mean distance from original
- **Full Unicode**: Extract all glyphs, not just ASCII
- **Centripetal knots**: Alternative parameterization option
- **Real font tests**: Smoke tests with actual fonts

## Dependencies

- `ttf-parser` - TTF/OTF parsing
- `serde` + `serde_json` - JSON serialization
- `clap` - CLI argument parsing

No heavyweight geometry libraries. All curve math is explicit and inspectable.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
