use clap::Parser;
use std::fs;
use std::path::PathBuf;
use solidus_geom::BSplineCurve;
use nurbie_font::NurbieGlyphData;

#[derive(Parser)]
#[command(name = "nurbie-mark")]
#[command(about = "Sample Nurbie B-splines into SVG polylines (microscopic marking prototype)", long_about = None)]
struct Args {
    /// Path to Nurbie glyph file (.nurbs.json)
    #[arg(long)]
    glyph: PathBuf,

    /// Output SVG file
    #[arg(long)]
    output: PathBuf,

    /// Number of samples per B-spline segment (default: 50)
    #[arg(long, default_value = "50")]
    samples: usize,
}

fn main() {
    let args = Args::parse();

    // Load Nurbie glyph
    let glyph_json = fs::read_to_string(&args.glyph)
        .expect("Failed to read glyph file");
    let glyph: NurbieGlyphData = serde_json::from_str(&glyph_json)
        .expect("Failed to parse glyph JSON");

    // Generate SVG
    let svg = generate_svg(&glyph, args.samples);

    // Write SVG
    fs::write(&args.output, svg)
        .expect("Failed to write SVG file");

    println!("✓ Generated SVG: {}", args.output.display());
    println!("  Font: {} ({})", glyph.font, glyph.glyph_name);
    println!("  Contours: {}", glyph.contours.len());
    println!("  Units per em: {}", glyph.units_per_em);
}

fn generate_svg(glyph: &NurbieGlyphData, samples_per_segment: usize) -> String {
    let units_per_em = glyph.units_per_em as f64;
    let advance_width = glyph.advance_width as f64;

    // SVG viewBox: flip Y (font space is Y-up, SVG is Y-down)
    // Add some padding
    let padding = units_per_em * 0.1;
    let view_width = advance_width + 2.0 * padding;
    let view_height = units_per_em + 2.0 * padding;

    let mut svg = String::new();
    svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg.push_str(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\"\n     viewBox=\"{} {} {} {}\"\n     width=\"800\" height=\"800\">\n",
        -padding, -units_per_em - padding, view_width, view_height));
    svg.push_str(&format!("  <desc>{} ({})</desc>\n\n", glyph.glyph_name, glyph.font));

    svg.push_str("  <!-- Background -->\n");
    svg.push_str(&format!("  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#f8f8f8\"/>\n\n",
        -padding, -units_per_em - padding, view_width, view_height));

    svg.push_str("  <!-- Baseline guide -->\n");
    svg.push_str(&format!("  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#ddd\" stroke-width=\"{}\"/>\n\n",
        -padding, 0.0, advance_width + padding, 0.0, units_per_em * 0.01));

    svg.push_str("  <!-- Em-height guide -->\n");
    svg.push_str(&format!("  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#ddd\" stroke-width=\"{}\"/>\n\n",
        -padding, -units_per_em, advance_width + padding, -units_per_em, units_per_em * 0.01));

    svg.push_str("  <!-- Advance width guide -->\n");
    svg.push_str(&format!("  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#ddd\" stroke-dasharray=\"4,4\" stroke-width=\"{}\"/>\n\n",
        advance_width, -units_per_em - padding, advance_width, padding, units_per_em * 0.005));

    // Draw contours
    for (i, contour) in glyph.contours.iter().enumerate() {
        let polyline = sample_bspline(contour, samples_per_segment);

        svg.push_str(&format!("  <!-- Contour {} -->\n", i));
        svg.push_str("  <polyline points=\"");

        for (j, point) in polyline.iter().enumerate() {
            if j > 0 {
                svg.push(' ');
            }
            // Flip Y for SVG (Y-down)
            svg.push_str(&format!("{:.2},{:.2}", point.x, -point.y));
        }

        svg.push_str("\" ");
        svg.push_str("fill=\"none\" ");
        svg.push_str("stroke=\"#000\" ");
        svg.push_str(&format!("stroke-width=\"{}\" ", units_per_em * 0.02));
        svg.push_str("stroke-linejoin=\"round\" ");
        svg.push_str("stroke-linecap=\"round\"/>\n");
    }

    svg.push_str("</svg>\n");
    svg
}

/// Sample a B-spline curve into a polyline
///
/// For v0: Simple uniform sampling in parameter space
/// Future: Adaptive sampling based on curvature
fn sample_bspline(curve: &BSplineCurve, samples_per_segment: usize) -> Vec<solidus_geom::Point2> {

    let n = curve.control_points.len();
    if n == 0 {
        return vec![];
    }

    // Number of segments = n - degree
    let num_segments = if n > curve.degree { n - curve.degree } else { 1 };
    let total_samples = num_segments * samples_per_segment;

    let mut points = Vec::with_capacity(total_samples + 1);

    for i in 0..=total_samples {
        let t = i as f64 / total_samples as f64;
        let point = evaluate_bspline(curve, t);
        points.push(point);
    }

    points
}

/// Evaluate B-spline at parameter t using de Boor's algorithm
///
/// Simplified for cubic (degree 3), non-rational curves
fn evaluate_bspline(curve: &BSplineCurve, t: f64) -> solidus_geom::Point2 {
    use solidus_geom::Point2;

    let n = curve.control_points.len();
    if n == 0 {
        return Point2::new(0.0, 0.0);
    }
    if n == 1 {
        return curve.control_points[0];
    }

    let degree = curve.degree;
    let knots = &curve.knots;

    // Clamp t to [0, 1]
    let t = t.clamp(0.0, 1.0);

    // Find the knot span
    let mut k = degree;
    for i in degree..knots.len() - degree - 1 {
        if t >= knots[i] && t < knots[i + 1] {
            k = i;
            break;
        }
    }

    // Handle t = 1.0 (end of curve)
    if t == 1.0 {
        k = knots.len() - degree - 2;
    }

    // de Boor's algorithm
    let mut d = vec![curve.control_points[k - degree..=k].to_vec()];

    for r in 1..=degree {
        let mut next_d = Vec::new();
        for i in 0..=(degree - r) {
            let ki = k - degree + r + i;
            let denominator = knots[ki + degree - r + 1] - knots[ki];

            // Handle zero denominator (knot multiplicity)
            let alpha = if denominator.abs() < 1e-10 {
                0.0  // When knots coincide, use first control point
            } else {
                (t - knots[ki]) / denominator
            };

            let p = d[r - 1][i].lerp(&d[r - 1][i + 1], alpha);
            next_d.push(p);
        }
        d.push(next_d);
    }

    d[degree][0]
}
