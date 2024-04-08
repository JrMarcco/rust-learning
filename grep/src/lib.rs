use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::ops::Range;
use std::path::Path;
use clap::Parser;

use colored::Colorize;
use itertools::Itertools;
use regex::Regex;

pub use error::GrepError;

mod error;

pub type StrategyFn = fn(&Path, &mut dyn BufRead, &Regex, &mut dyn Write) -> Result<(), GrepError>;

#[derive(Parser, Debug)]
#[clap(version = "1.0.0", author = "jrmarcco")]
pub struct GrepConfig {
    pattern: String,
    glob: String,
}

impl GrepConfig {
    pub fn match_with_strategy(&self) -> Result<(), GrepError> {
        self.match_with(default_strategy)
    }

    pub fn match_with(&self, strategy: StrategyFn) -> Result<(), GrepError> {
        let regex = Regex::new(&self.pattern)?;
        let files: Vec<_> = glob::glob(&self.glob)?.collect();


        files.into_iter().for_each(|v| {
            if let Ok(filename) = v {
                if let Ok(file) = File::open(&filename) {
                    let mut reader = BufReader::new(file);
                    let mut stdout = io::stdout();

                    if let Err(e) = strategy(filename.as_path(), &mut reader, &regex, &mut stdout) {
                        println!("Internal error: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

pub fn default_strategy(path: &Path, reader: &mut dyn BufRead, pattern: &Regex, writer: &mut dyn Write) -> Result<(), GrepError> {
    let matches: String = reader.lines()
        .enumerate()
        .map(|(lineno, line)| {
            line.ok().map(|line| {
                pattern
                    .find(&line)
                    .map(|m| format_line(&line, lineno + 1, m.range()))
            }).flatten()
        })
        .filter_map(|v| v.ok_or(()).ok())
        .join("\n");

    if !matches.is_empty() {
        writer.write_all(path.display().to_string().green().as_bytes())?;
        writer.write_all(b"\n")?;
        writer.write_all(matches.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}

pub fn format_line(line: &str, lineno: usize, range: Range<usize>) -> String {
    let Range { start, end } = range;
    let prefix = &line[..start];

    format!(
        "{0: >6}:{1: <3} {2}{3}{4}",
        lineno.to_string().blue(),
        (prefix.chars().count() + 1).to_string().cyan(),
        prefix,
        &line[start..end].red(),
        &line[end..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_line_should_work() {
        let result = format_line("Hello, jrmarcco !", 1000, 7..10);
        let expected = format!(
            "{0: >6}:{1: <3} Hello, {2} !",
            "1000".blue(),
            "8".cyan(),
            "jrmarcco".red(),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn default_strategy_should_work() {
        let path = Path::new("src/main.rs");
        let input = b"hello world!\nhey jrmarcco!";
        let mut reader = BufReader::new(&input[..]);
        let pattern = Regex::new(r"he\w+").unwrap();
        let mut writer = Vec::new();
        default_strategy(path, &mut reader, &pattern, &mut writer).unwrap();

        let result = String::from_utf8(writer).unwrap();
        let expected = [
            String::from("src/main.rs"),
            format_line("hello world!", 1, 0..5),
            format_line("hey jrmarcco!\n", 2, 0..3),
        ];

        assert_eq!(result, expected.join("\n"));
    }
}
