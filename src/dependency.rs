use crate::version::{pick_version, version_requirements};
use anyhow::{Context, Result, bail};
use lief::Binary;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::resolve::{resolve_library};

#[derive(Clone, Debug, Serialize)]
pub struct Dependency {
    pub name: String,
    pub deps: Vec<Dependency>,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
}

impl Dependency {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        Self::from_file_inner(path, &mut Vec::new())
    }

    fn from_file_inner(path: &PathBuf, seen: &mut Vec<PathBuf>) -> Result<Self> {
        let binary = Binary::parse(path)
            .with_context(|| format!("failed to parse binary file: {:?}", path))?;

        // (lib_name, version) pairs
        let lib_entries: Vec<(String, Option<String>)> = match binary {
            Binary::ELF(ref elf) => {
                let versions = version_requirements(&elf);
                elf.dynamic_entries()
                    .filter_map(|entry| match entry {
                        lief::elf::dynamic::Entries::Library(lib) => {
                            let name = lib.name().to_string();
                            let version = versions.get(&name).and_then(|v| pick_version(v));
                            Some((name, version))
                        }
                        _ => None,
                    })
                    .collect()
            }
            Binary::PE(ref pe) => {
                let mut seen = HashMap::new();
                for imp in pe.imports() {
                    let name = imp.name().to_string();
                    let key = name.to_ascii_lowercase();
                    seen.entry(key).or_insert(name);
                }
                seen.into_values().map(|name| (name, None)).collect()
            }
            Binary::MachO(ref fat) => {
                let macho = fat.iter().next().context("empty Mach-O FAT container")?;
                macho
                    .libraries()
                    .map(|lib| {
                        let (maj, min, patch) = lib.current_version();
                        (lib.name().to_string(), Some(format!("{maj}.{min}.{patch}")))
                    })
                    .collect()
            }
            Binary::COFF(_) => bail!("COFF binaries are not supported yet"),
        };

        // avoid infinite loops
        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
        if seen.contains(&canonical) {
            return Ok(Dependency {
                name: path_name(path),
                deps: vec![],
                path: Some(path.clone()),
                version: None,
            });
        }
        seen.push(canonical);

        let mut deps = Vec::with_capacity(lib_entries.len());
        for (lib_name, version) in lib_entries {
            let dep = match resolve_library(&lib_name, path, &binary) {
                Some(resolved) => {
                    let mut d = Self::from_file_inner(&resolved, seen)?;
                    d.version = pe_file_version(&resolved).or(version).or(d.version);
                    d
                }
                None => Dependency {
                    name: lib_name,
                    deps: vec![],
                    path: None,
                    version,
                },
            };
            deps.push(dep);
        }

        Ok(Dependency {
            name: path_name(path),
            deps,
            path: Some(path.clone()),
            version: None,
        })
    }
}

fn path_name(path: &PathBuf) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}
fn pe_file_version(path: &PathBuf) -> Option<String> {
    None // TODO real implementation
}
