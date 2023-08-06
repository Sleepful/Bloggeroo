// use clap::{builder::IntoResettable, Parser};
use clap::Parser;
use glob::glob;
use regex::Regex;
// use std::env::current_dir;
// use std::path::PathBuf;
use anyhow::anyhow;
use anyhow::{Error, Result};
use markdown::to_html_with_options;
use markdown::to_mdast;
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
#[derive(Debug)]
enum ParseMd {
    Result((markdown::mdast::Node, std::string::String)),
}

fn get_frontmatter_value(ast: &markdown::mdast::Node) -> Option<String> {
    // frontmatter should always be at the front[0]
    match &ast.children()?[0] {
        markdown::mdast::Node::Yaml(yaml) => Some(yaml.value.clone()),
        _ => return None,
    }
}

fn article_yaml(value: &String, match_on: &str) -> Option<String> {
    let re_string = format!(r"[n]?{}:\s+(.+)[n]?", match_on);
    let re = Regex::new(&re_string).unwrap();
    let publish = re.captures(value)?[1].to_string();
    // println!("captures {:?}", re.captures(value)?);
    Some(publish)
}

fn article_publish(fm_string: &String) -> Option<String> {
    article_yaml(fm_string, "publish")
}
fn article_uuid(fm_string: &String) -> Option<String> {
    article_yaml(fm_string, "uuid")
}
fn article_date(fm_string: &String) -> Option<String> {
    article_yaml(fm_string, "date")
}

fn article_title(fm_string: &String) -> Option<String> {
    article_yaml(fm_string, "title")
}

struct Article {
    title: String,
    uuid: String,
    date: String,
    publish: bool,
    html: String,
    dir: PathBuf,
}

fn create_article(dir: PathBuf) -> Option<Article> {
    let md = std::fs::read_to_string(dir.clone()).ok()?;
    let ParseMd::Result((ast, html)) = parse_md(&md[..]).ok()?;
    let fm = get_frontmatter_value(&ast)?;
    let date = article_date(&fm)?;
    let uuid = article_uuid(&fm)?;
    let title = article_title(&fm)?;
    let publish = article_publish(&fm)? == "true";
    Some(Article {
        uuid,
        date,
        publish,
        title,
        html,
        dir,
    })
}

// will write it if all the YAML fns return Ok()
fn write_html(ast: &markdown::mdast::Node, out_dir: String) {}

fn parse_md(md: &str) -> anyhow::Result<ParseMd, String> {
    let custom = || Constructs {
        frontmatter: true,
        ..Constructs::gfm()
    };
    let parse_options = || {
        return ParseOptions {
            constructs: custom(),
            ..ParseOptions::gfm()
        };
    };
    let options = || Options {
        parse: parse_options(),
        ..Options::gfm()
    };
    let ast = to_mdast(md, &(parse_options()))?;
    // let fm = get_frontmatter_value(&ast).ok_or("eh")?;
    // println!("frontmatter: {:?}", fm);
    // let date = article_date(&fm);
    // let date = article_yaml(&fm, "date");
    // println!("date: {:?}", date);
    let html = to_html_with_options(md, &(options()))?;
    Ok(ParseMd::Result((ast, html)))
}

fn main() {
    let args = Args::parse();
    println!("Hello {}!", args.input_dir);
    println!("Hello {}!", args.output_dir);
    // glob
    let files = find_md_files(&args.input_dir[..]);
    if files.is_empty() {
        println!("Zero markdown files were found.");
        exit(1)
    }
    println!("files {:#?}!", files);
    // markdown rs
    // read files to memory
    let htmls: Vec<Article> = files.into_iter().flat_map(|f| create_article(f)).collect();

    // let html = produce_html_from_md(&"blabla"[..]);
    // write files to output dir
    println!(
        "titles: {:?}",
        htmls
            .into_iter()
            .map(|a| -> String { a.title })
            .collect::<String>()
    )
    // tera
}
