use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Dependency {
    pub name: String,
    pub deps: Vec<Dependency>,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
}