use anyhow::Result;
use clap::Parser;
use grep::GrepConfig;

fn main() -> Result<()> {
    let config: GrepConfig = GrepConfig::parse();
    config.match_with_strategy()?;

    Ok(())
}