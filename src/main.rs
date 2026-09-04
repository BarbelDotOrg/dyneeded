pub mod serialize;
pub mod dependency;
pub mod result;
pub mod version;

use std::path::PathBuf;
use clap::Parser;
use lief::Binary;
use crate::dependency::Dependency;

fn validate_file_exists(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("'{}' is not a valid file", s))
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Ldd,
    Text,
    Json,
    Ron,
    Tree,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_parser = validate_file_exists)]
    file: PathBuf,

    #[arg(short, long, value_enum, default_value = "text")]
    format: OutputFormat,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let tree = Dependency::from_file(&args.file)?;
    println!("{}", tree.serialize(args.format)?);
    Ok(())
}