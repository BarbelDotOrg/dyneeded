use crate::version::{pick_version, version_requirements};
use anyhow::{Context, Result, bail};
use lief::Binary;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

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
            Binary::ELF(elf) => {
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
            Binary::PE(pe) => {
                let mut seen = HashMap::new();
                for imp in pe.imports() {
                    let name = imp.name().to_string();
                    let key = name.to_ascii_lowercase();
                    seen.entry(key).or_insert(name);
                }
                seen.into_values().map(|name| (name, None)).collect()
            }
            Binary::MachO(fat) => {
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
        for (lib_name, _version) in lib_entries {
            let dep = match resolve_library(&lib_name, path) {
                Some(resolved) => {
                    let mut d = Self::from_file_inner(&resolved, seen)?;
                    d.version = pe_file_version(&resolved).or(d.version);
                    d
                }
                None => Dependency {
                    name: lib_name,
                    deps: vec![],
                    path: None,
                    version: None,
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

/// Resolve a bare library name (e.g. "libc.so.6") to an actual path on disk,
/// the way a real loader would (search paths, rpath, etc).
fn resolve_library(name: &str, _requesting_binary: &PathBuf) -> Option<PathBuf> {
    // Placeholder: real impl should check rpath/runpath from the ELF
    // dynamic entries, LD_LIBRARY_PATH, then standard dirs like
    // /lib, /usr/lib, /lib64, etc. (or PATH / System32 for PE).
    for dir in ["/lib", "/lib64", "/usr/lib", "/usr/lib64"] {
        let candidate = PathBuf::from(dir).join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn pe_file_version(path: &PathBuf) -> Option<String> {
    let binary = Binary::parse(path)?;
    let Binary::PE(pe) = binary else { return None };
    let resources = pe.resources_manager()?;
    let version = resources.version().next()?; // <-- was the bug: .next() on the iterator

    // The current LIEF (0.17+/1.0) API replaced the old has_fixed_file_info()/
    // fixed_file_info() pair with a non-optional file_info() accessor.
    let info = version.file_info();
    Some(format!(
        "{}.{}.{}.{}",
        info.file_version_ms >> 16,
        info.file_version_ms & 0xFFFF,
        info.file_version_ls >> 16,
        info.file_version_ls & 0xFFFF,
    ))
}
