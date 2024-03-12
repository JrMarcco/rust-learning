use std::collections::HashMap;
use std::str::FromStr;
use clap::Parser;
use anyhow::{anyhow, Result};
use colored::Colorize;
use mime::Mime;
use reqwest::{Client, header, Response, Url};
use reqwest::header::HeaderMap;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[derive(Parser, Debug)]
#[clap(version = "1.0.0", author = "jrmarcco")]
struct Opts {
    #[clap(subcommand)]
    sub_cmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post),
}

#[derive(Parser, Debug)]
struct Get {
    #[arg(value_parser = parse_url)]
    url: String,
}

#[derive(Parser, Debug)]
struct Post {
    #[arg(value_parser = parse_url)]
    url: String,
    #[arg(value_parser = parse_kv_pair)]
    body: Vec<KvPair>,
}

#[derive(Debug, Clone, PartialEq)]
struct KvPair {
    key: String,
    val: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));

        Ok(Self {
            key: split.next().ok_or_else(err)?.to_string(),
            val: split.next().ok_or_else(err)?.to_string(),
        })
    }
}

fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?;
    return Ok(s.into());
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    return Ok(s.parse()?);
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let rsp = client.get(&args.url).send().await?;
    return Ok(print_rsp(rsp).await?);
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.key, &pair.val);
    }

    let rsp = client.post(&args.url).json(&body).send().await?;
    return Ok(print_rsp(rsp).await?);
}

fn print_status(rsp: &Response) {
    let status = format!("{:?} {}", rsp.version(), rsp.status()).blue();
    println!("{}\n", status);
}

fn print_headers(rsp: &Response) {
    for (name, val) in rsp.headers() {
        println!("{}: {:?}", name, val)
    }

    println!();
}

fn print_syntect(s: &str, ext: &str) {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension(ext).unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    for line in LinesWithEndings::from(s) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}", escaped);
    }
}

fn get_content_type(rsp: &Response) -> Option<Mime> {
    return rsp.headers().get(header::CONTENT_TYPE).map(|v| v.to_str().unwrap().parse().unwrap());
}

fn print_body(m: Option<Mime>, body: &str) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => print_syntect(body, "json"),
        Some(v) if v == mime::TEXT_HTML => print_syntect(body, "html"),

        _ => println!("{}", body),
    }
}

async fn print_rsp(rsp: Response) -> Result<()> {
    print_status(&rsp);
    print_headers(&rsp);

    let mime = get_content_type(&rsp);
    let body = rsp.text().await?;
    print_body(mime, &body);

    return Ok(());
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let mut headers = HeaderMap::new();
    headers.insert("X-POWERED-BY", "Rust".parse()?);
    headers.insert(header::USER_AGENT, "Rust Httpie".parse()?);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let res = match opts.sub_cmd {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) =>post(client, args).await?,
    };
    println!("{:?}", res);

    return Ok(());
}
