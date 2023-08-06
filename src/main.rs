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
    // re.captures()?[0] contains the matched string
    // re.captures()?[1] contains the capture group
    let publish = re.captures(value)?[1].to_string();
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
    input_dir: PathBuf,
}

struct WrittenArticle {
    article: Article,
    file_path: PathBuf,
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
        input_dir: dir,
    })
}

// will write it if all the YAML fns return Ok()
fn write_html(article: Article, out_dir: &str) -> Result<WrittenArticle, Error> {
    if let true = article.publish {
        let file_name = format!("{}.html", article.uuid);
        let mut path = PathBuf::new();
        path.push(out_dir);
        path.push("articles");
        std::fs::create_dir(&path)?;
        path.push(file_name);
        std::fs::write(&path, &article.html)?;
        let w = WrittenArticle {
            article,
            file_path: path,
        };
        Ok(w)
    } else {
        Err(anyhow!(format!("Could not write file {}", article.uuid)))
    }
}

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
    let html = to_html_with_options(md, &(options()))?;
    Ok(ParseMd::Result((ast, html)))
}

fn main() {
    let args = Args::parse();
    println!("Hello {}!", args.input_dir);
    println!("Hello {}!", args.output_dir);
    // glob
    let files = find_md_files(&args.input_dir);
    if files.is_empty() {
        println!("Zero markdown files were found.");
        exit(1)
    }
    println!("files {:#?}!", files);
    // markdown rs
    // read files to memory
    let htmls: Vec<WrittenArticle> = files
        .into_iter()
        .flat_map(|f| {
            let art = create_article(f).ok_or(anyhow!("Create article fail"))?;
            write_html(art, &args.output_dir)
        })
        .collect();

    println!(
        "titles: {:?}",
        htmls
            .into_iter()
            .map(|a| -> PathBuf { a.file_path })
            .collect::<PathBuf>()
    )
    // tera
}
