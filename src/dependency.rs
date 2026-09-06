use crate::resolve::resolve_library;
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

        // Extract library imports and their versions
        let lib_entries: Vec<(String, Option<String>)> = match binary {
            Binary::ELF(ref elf) => extract_elf_versions(elf),
            Binary::PE(ref pe) => extract_pe_versions(pe),
            Binary::MachO(ref fat) => extract_macho_versions(fat)?,
            Binary::COFF(_) => bail!("COFF binaries are not supported yet"),
        };


        let self_version = extract_file_version(&binary);

        // no infinite loops
        // how can protestants know their canon?
        // easy, just use .canonicalize() in rust...
        // filthy papists...
        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
        if seen.contains(&canonical) {
            return Ok(Dependency {
                name: path_name(path),
                deps: vec![],
                path: Some(path.clone()),
                version: self_version,
            });
        }
        seen.push(canonical);

        let mut deps = Vec::with_capacity(lib_entries.len());
        for (lib_name, import_version) in lib_entries {
            let dep = match resolve_library(&lib_name, path, &binary) {
                Some(resolved) => {
                    let mut d = Self::from_file_inner(&resolved, seen)?;
                    // fallback order: child version -> import constraint -> pre-existing child version
                    d.version = d.version.or(import_version);
                    d
                }
                None => Dependency {
                    name: lib_name,
                    deps: vec![],
                    path: None,
                    version: import_version,
                },
            };
            deps.push(dep);
        }

        Ok(Dependency {
            name: path_name(path),
            deps,
            path: Some(path.clone()),
            version: self_version,
        })
    }
}

fn path_name(path: &PathBuf) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn extract_elf_versions(elf: &lief::elf::Binary) -> Vec<(String, Option<String>)> {
    let versions = version_requirements(elf);
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

fn extract_pe_versions(pe: &lief::pe::Binary) -> Vec<(String, Option<String>)> {
    let mut seen = HashMap::new();
    const PREFIXES_TO_SKIP: &[&str] = &["api-ms", "ext-ms"];

    for imp in pe.imports() {
        let name = imp.name().to_string();
        let key = name.to_ascii_lowercase();

        if PREFIXES_TO_SKIP.iter().any(|prefix| key.starts_with(prefix)) {
            continue;
        }

        seen.entry(key).or_insert(name);
    }
    seen.into_values().map(|name| (name, None)).collect()
}

fn extract_macho_versions(fat: &lief::macho::FatBinary) -> Result<Vec<(String, Option<String>)>> {
    let macho = fat.iter().next().context("empty Mach-O FAT container")?;
    Ok(macho
        .libraries()
        .map(|lib| {
            let (maj, min, patch) = lib.current_version();
            (lib.name().to_string(), Some(format!("{maj}.{min}.{patch}")))
        })
        .collect())
}

fn extract_file_version(binary: &Binary) -> Option<String> {
    match binary {
        Binary::PE(pe) => extract_pe_file_version(pe),
        Binary::MachO(macho) => extract_macho_file_version(macho),
        Binary::ELF(elf) => extract_elf_file_version(elf),
        _ => None,
    }
}

fn extract_pe_file_version(pe: &lief::pe::Binary) -> Option<String> {
    let mgr = pe.resources_manager()?;
    let info = mgr.version().into_iter().next()?;
    let fixed = info.file_info();

    let ms = fixed.file_version_ms;
    let ls = fixed.file_version_ls;

    let major = (ms >> 16) & 0xFFFF; // league soccer
    let minor = ms & 0xFFFF;
    let build = (ls >> 16) & 0xFFFF;
    let revision = ls & 0xFFFF;

    Some(format!("{major}.{minor}.{build}.{revision}"))
}

fn extract_macho_file_version(macho: &lief::macho::FatBinary) -> Option<String> {
    None // TODO
}

fn extract_elf_file_version(elf: &lief::elf::Binary) -> Option<String> {
    None // TODO
}