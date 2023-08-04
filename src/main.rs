// use clap::{builder::IntoResettable, Parser};
use clap::Parser;
use glob::glob;
// use std::env::current_dir;
// use std::path::PathBuf;
use anyhow::anyhow;
use anyhow::Error;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory, use a glob pattern,
    /// example:
    ///   -i '/Users/me/Publish/**/*'
    /// Remember to use single quotes such that
    /// your shell doesn't expand the glob pattern.
    /// Bloggeroo will find all the files with an
    /// `.md` extension.
    #[arg(short, long)]
    input_dir: String,

    /// Output directory, here goes your html.
    #[arg(short, long)]
    output_dir: String,
}

fn find_md_files(input_dir: &str) -> Vec<PathBuf> {
    glob(input_dir)
        .expect("Failed to read input_dir")
        .into_iter()
        .filter_map(|s| s.ok())
        .filter(|s| s.extension().is_some_and(|x| x == "md"))
        .collect()
}

fn main() {
    let args = Args::parse();
    println!("Hello {}!", args.input_dir);
    println!("Hello {}!", args.output_dir);
    let files = find_md_files(&args.input_dir[..]);
    // glob
    // markdown rs
}
