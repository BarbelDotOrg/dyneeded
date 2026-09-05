pub mod bible;
pub mod dependency;
pub mod serialize;
pub mod version;

use crate::bible::random_bible_passage;
use crate::dependency::Dependency;
use clap::Parser;
use std::path::PathBuf;

fn validate_optional_file_exists(s: &str) -> Result<PathBuf, String> {
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
    #[arg(
        value_parser = validate_optional_file_exists,
        required_unless_present = "bible",
    help = "The file to analyze"
    )]
    file: Option<PathBuf>,

    #[arg(short = 'b', long = "bible", help = "Display a bible quote")]
    bible: bool,

    #[arg(
        short,
        long,
        value_enum,
        default_value = "text",
        help = "Output format"
    )]
    format: OutputFormat,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.bible {
        println!("{}", random_bible_passage())
    } else if let Some(file) = args.file {
        let tree = Dependency::from_file(&file)?;
        println!("{}", tree.serialize(args.format)?);
    }

    Ok(())
}
