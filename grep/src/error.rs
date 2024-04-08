use thiserror::Error;

#[derive(Error, Debug)]
pub enum GrepError {
    #[error("Global pattern error")]
    GlobalPatternErr(#[from] glob::PatternError),
    #[error("Regex pattern error")]
    RegexPatternErr(#[from] regex::Error),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
}