// use clap::{builder::IntoResettable, Parser};
use clap::Parser;
use glob::glob;
// use std::env::current_dir;
// use std::path::PathBuf;
use anyhow::anyhow;
use anyhow::Error;
use markdown::to_html_with_options;
use markdown::Constructs;
use markdown::Options;
use markdown::ParseOptions;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

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

// TODO:
// - read files from dirs
// - prodce html for each file
// - write to each file out
// - get filename properly from...metadata?
fn produce_html_from_md() {
    let custom = Constructs {
        frontmatter: true,
        ..Constructs::gfm()
    };
    let options = Options {
        parse: ParseOptions {
            constructs: custom,
            ..ParseOptions::gfm()
        },
        ..Options::gfm()
    };
    let md = to_html_with_options(&"blabla"[..], &options);

    println!("md {:#?}!", md);
}

fn main() {
    let args = Args::parse();
    println!("Hello {}!", args.input_dir);
    println!("Hello {}!", args.output_dir);
    // glob
    let files = find_md_files(&args.input_dir[..]);
    if let 0 = files.len() {
        println!("Zero markdown files were found.");
        exit(1)
    }
    println!("files {:#?}!", files);
    // markdown rs
    produce_html_from_md()
    // tera
}
