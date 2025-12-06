use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nurbie-font")]
#[command(about = "Convert primitive segments to cubic B-splines", long_about = None)]
struct Args {
    /// Path to Casteljau directory (input)
    #[arg(long)]
    casteljau_dir: PathBuf,

    /// Output directory for Nurbie B-splines
    #[arg(long)]
    out_dir: PathBuf,
}

fn main() {
    let args = Args::parse();

    if let Err(e) = nurbie_font::convert_font(&args.casteljau_dir, &args.out_dir) {
        eprintln!("Error converting font: {}", e);
        std::process::exit(1);
    }

    println!(
        "Successfully converted font to {}",
        args.out_dir.display()
    );
}
