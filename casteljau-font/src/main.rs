use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "casteljau-font")]
#[command(about = "Extract font glyph outlines to primitive segments", long_about = None)]
struct Args {
    /// Path to input TTF/OTF font file
    #[arg(long)]
    font: PathBuf,

    /// Output directory for Casteljau primitives
    #[arg(long)]
    out_dir: PathBuf,

    /// Filter to specific glyphs (e.g., "U+0041" or "U+0041,U+0053")
    /// If not specified, extracts ASCII printable range (U+0020-U+007E)
    #[arg(long)]
    glyph_filter: Option<String>,
}

fn main() {
    let args = Args::parse();

    // Parse glyph filter if provided
    let glyph_filter = args.glyph_filter.as_ref().map(|filter| {
        filter
            .split(',')
            .filter_map(|s| {
                let s = s.trim();
                if s.starts_with("U+") {
                    u32::from_str_radix(&s[2..], 16).ok()
                } else {
                    None
                }
            })
            .collect::<Vec<u32>>()
    });

    if let Err(e) = casteljau_font::extract_font_filtered(&args.font, &args.out_dir, glyph_filter.as_deref()) {
        eprintln!("Error extracting font: {}", e);
        std::process::exit(1);
    }

    println!(
        "Successfully extracted font to {}",
        args.out_dir.display()
    );
}
