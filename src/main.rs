// use clap::{builder::IntoResettable, Parser};
#[macro_use]
extern crate lazy_static;
use anyhow::anyhow;
use anyhow::{Error, Result};
use chrono::NaiveDateTime;
use clap::Parser;
use glob::glob;
use markdown::to_html_with_options;
use markdown::to_mdast;
use markdown::Constructs;
use markdown::Options;
use markdown::ParseOptions;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::exit;
use tera::Tera;

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
        _ => None,
    }
}

fn article_yaml(value: &str, match_on: &str) -> Option<String> {
    let re_string = format!(r"[n]?{}:\s+(.+)[n]?", match_on);
    let re = Regex::new(&re_string).unwrap();
    // re.captures()?[0] contains the matched string
    // re.captures()?[1] contains the capture group
    let publish = re.captures(value)?[1].to_string();
    Some(publish)
}

fn article_publish(fm_string: &str) -> Option<String> {
    article_yaml(fm_string, "publish")
}
fn article_uuid(fm_string: &str) -> Option<String> {
    article_yaml(fm_string, "uuid")
}
fn article_date(fm_string: &str) -> Option<String> {
    article_yaml(fm_string, "date")
}

fn article_title(fm_string: &str) -> Option<String> {
    article_yaml(fm_string, "title")
}

#[derive(Serialize, Deserialize, Debug)]
struct Written {
    file_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
struct Article {
    title: String,
    uuid: String,
    pretty_date: String,
    date: String,
    publish: bool,
    html: String,
    input_dir: PathBuf,
    written: Option<Written>,
}

struct Index {
    articles: Vec<Article>,
}

fn create_article(dir: PathBuf) -> Result<Article, Error> {
    let md = std::fs::read_to_string(dir.clone())?;
    let ParseMd::Result((ast, html)) = parse_md(&md[..]).expect("parsemd failed");
    let fm = get_frontmatter_value(&ast).unwrap();
    let date = article_date(&fm).unwrap();
    let uuid = article_uuid(&fm).unwrap();
    let title = article_title(&fm).unwrap();
    let publish = article_publish(&fm).unwrap_or_default() == "true";
    let pretty_date = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", &date), "%Y-%m-%d %T")
        .unwrap_or_default()
        .format("%Y, %B %e")
        .to_string();
    Ok(Article {
        uuid,
        date,
        pretty_date,
        publish,
        title,
        html,
        input_dir: dir,
        written: None,
    })
}

lazy_static! {
    pub static ref TEMPLATES: Tera = parse_tera();
}

fn parse_tera() -> Tera {
    match Tera::new("templates/**/*.html") {
        Ok(tera) => tera,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    }
}

fn build_article_context(art: &Article) -> tera::Context {
    let mut context = tera::Context::new();
    context.insert("article", art);
    context
}

fn build_index_context(idx: &Index) -> tera::Context {
    let mut context = tera::Context::new();
    context.insert("articles", &idx.articles);
    context
}

fn render_template(template: &str, ctx: tera::Context) -> Result<String, Error> {
    let rendered = TEMPLATES.render(template, &ctx)?;
    Ok(rendered)
}

fn render_article_template(ctx: tera::Context) -> Result<String, Error> {
    let html = render_template("article.html", ctx)?;
    Ok(html)
}

fn render_index_template(ctx: tera::Context) -> Result<String, Error> {
    let html = render_template("index.html", ctx)?;
    Ok(html)
}

fn write_index(index: &Index, out_dir: &str) -> Result<PathBuf, Error> {
    let file_name = "index.html";
    let mut path = PathBuf::new();
    path.push(out_dir);
    path.push("html");
    path.push(file_name);
    let ctx = build_index_context(index);
    let html = render_index_template(ctx)?;
    std::fs::write(&path, html)?;
    Ok(path)
}

// will write it if all the YAML fns return Ok()
fn write_article(mut article: Article, out_dir: &str) -> Result<Article, Error> {
    if article.publish {
        let file_name = format!("{}.html", article.uuid);
        let mut path = PathBuf::new();
        path.push(out_dir);
        path.push("html");
        path.push("articles");
        std::fs::create_dir_all(&path).expect("Failed creating html dirs");
        path.push(file_name);
        let ctx = build_article_context(&article);
        let html = render_article_template(ctx).expect("Could not render article template");
        // std::fs::write(&path, &article.html)?;
        std::fs::write(&path, html)?;
        article.written = Some(Written { file_path: path });
        Ok(article)
    } else {
        Err(anyhow!(format!("Could not write file {}", article.uuid)))
    }
}

fn parse_md(md: &str) -> anyhow::Result<ParseMd, String> {
    let custom = Constructs {
        frontmatter: true,
        ..Constructs::gfm()
    };
    let parse_options = ParseOptions {
        constructs: custom,
        ..ParseOptions::gfm()
    };
    let options = Options {
        parse: parse_options,
        ..Options::gfm()
    };
    let ast = to_mdast(md, &options.parse)?;
    let html = to_html_with_options(md, &options)?;
    Ok(ParseMd::Result((ast, html)))
}

fn main() {
    let args = Args::parse();
    // glob
    let files = find_md_files(&args.input_dir);
    if files.is_empty() {
        println!("Zero markdown files were found.");
        exit(1)
    }
    println!("Files found:\n {:#?}", files);
    let articles: Vec<Article> = files
        .into_iter()
        .flat_map(|f| {
            // read file to memory
            let art = create_article(f).expect("Create article fail");
            // and write it with tera templates
            write_article(art, &args.output_dir)
        })
        .collect();
    let index = Index { articles };
    write_index(&index, &args.output_dir).expect("Could not write index.html");

    println!(
        "Articles created:\n {:#?}",
        index
            .articles
            .into_iter()
            .map(|a| -> PathBuf { a.written.unwrap().file_path })
            .collect::<Vec<PathBuf>>()
    );
}
