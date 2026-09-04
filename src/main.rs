pub mod analysis;

use std::path::PathBuf;
use clap::Parser;
use lief::Binary;

fn validate_file_exists(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("'{}' is not a valid file", s))
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Ldd,
    Text,
    Json,
    Tree,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_parser = validate_file_exists)]
    file: PathBuf,

    #[arg(short, long, value_enum, default_value = "text")]
    format: OutputFormat,
}

fn main() {
    let args = Args::parse();
}