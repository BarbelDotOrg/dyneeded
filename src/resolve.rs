use std::env;
use std::path::{Path, PathBuf};

use lief::elf::dynamic::Entries as ElfDynEntries;
use lief::macho::commands::Commands as MachOCommands;
use lief::Binary;

pub fn resolve_library(
    name: &str,
    requesting_binary_path: &PathBuf,
    requesting_binary: &Binary,
) -> Option<PathBuf> {
    match requesting_binary {
        Binary::ELF(elf_bin) => resolve_elf(name, requesting_binary_path, elf_bin),
        Binary::PE(_pe_bin) => resolve_pe(name, requesting_binary_path),
        Binary::MachO(fat_bin) => {
            let macho_bin = fat_bin.iter().next()?;
            resolve_macho(name, requesting_binary_path, &macho_bin)
        }
        _ => None,
    }
}

fn resolve_elf(name: &str, bin_path: &Path, elf_bin: &lief::elf::Binary) -> Option<PathBuf> {
    if name.contains('/') {
        let candidate = PathBuf::from(name);
        return candidate.is_file().then_some(candidate);
    }

    let origin = bin_path.parent().unwrap_or_else(|| Path::new("."));

    let mut rpaths: Vec<String> = Vec::new();
    let mut runpaths: Vec<String> = Vec::new();

    for entry in elf_bin.dynamic_entries() {
        match entry {
            ElfDynEntries::Rpath(rpath) => {
                rpaths.extend(rpath.paths().iter().map(|p| p.to_string()));
            }
            ElfDynEntries::RunPath(runpath) => {
                runpaths.extend(runpath.paths().iter().map(|p| p.to_string()));
            }
            _ => {}
        }
    }

    let expand_origin = |raw: &str| -> PathBuf {
        let origin_str = origin.to_string_lossy();
        let expanded = raw
            .replace("$ORIGIN", &origin_str)
            .replace("${ORIGIN}", &origin_str);
        PathBuf::from(expanded)
    };

    if runpaths.is_empty() {
        for rp in &rpaths {
            let candidate = expand_origin(rp).join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    if let Ok(ld_library_path) = env::var("LD_LIBRARY_PATH") {
        for dir in env::split_paths(&ld_library_path) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    for rp in &runpaths {
        let candidate = expand_origin(rp).join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    for dir in [
        "/lib",
        "/lib64",
        "/usr/lib",
        "/usr/lib64",
        "/usr/local/lib",
        "/usr/local/lib64",
    ] {
        let candidate = PathBuf::from(dir).join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

fn resolve_pe(name: &str, bin_path: &Path) -> Option<PathBuf> {
    let app_dir = bin_path.parent().unwrap_or_else(|| Path::new("."));

    if let Some(found) = find_case_insensitive(app_dir, name) {
        return Some(found);
    }

    for dir in system_dirs() {
        if let Some(found) = find_case_insensitive(&dir, name) {
            return Some(found);
        }
    }

    if let Some(sysroot) = env::var_os("SystemRoot").or_else(|| env::var_os("WINDIR")) {
        if let Some(found) = find_case_insensitive(Path::new(&sysroot), name) {
            return Some(found);
        }
    }

    if let Ok(cwd) = env::current_dir() {
        if let Some(found) = find_case_insensitive(&cwd, name) {
            return Some(found);
        }
    }

    if let Ok(path_var) = env::var("PATH") {
        for dir in env::split_paths(&path_var) {
            if let Some(found) = find_case_insensitive(&dir, name) {
                return Some(found);
            }
        }
    }

    None
}

fn system_dirs() -> Vec<PathBuf> {
    let root = env::var_os("SystemRoot")
        .or_else(|| env::var_os("WINDIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\Windows"));
    vec![root.join("System32"), root.join("SysWOW64")]
}

fn find_case_insensitive(dir: &Path, name: &str) -> Option<PathBuf> {
    let direct = dir.join(name);
    if direct.is_file() {
        return Some(direct);
    }
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        if entry.file_name().to_string_lossy().eq_ignore_ascii_case(name) {
            let path = entry.path();
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

fn resolve_macho(name: &str, bin_path: &Path, macho_bin: &lief::macho::Binary) -> Option<PathBuf> {
    let loader_path = bin_path.parent().unwrap_or_else(|| Path::new("."));
    let executable_path = loader_path;

    let expand = |raw: &str| -> PathBuf {
        let loader_str = loader_path.to_string_lossy();
        let exe_str = executable_path.to_string_lossy();
        let expanded = raw
            .replace("@loader_path", &loader_str)
            .replace("@executable_path", &exe_str);
        PathBuf::from(expanded)
    };

    let basename = name.rsplit('/').next().unwrap_or(name);

    if let Ok(dyld_library_path) = env::var("DYLD_LIBRARY_PATH") {
        for dir in env::split_paths(&dyld_library_path) {
            let candidate = dir.join(basename);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    if let Some(suffix) = name.strip_prefix("@rpath/") {
        for cmd in macho_bin.commands() {
            if let MachOCommands::RPath(rpath_cmd) = cmd {
                let candidate = expand(&rpath_cmd.path()).join(suffix);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    } else if name.starts_with("@loader_path/") || name.starts_with("@executable_path/") {
        let candidate = expand(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    } else if name.contains('/') {
        let candidate = PathBuf::from(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    } else {
        let candidate = loader_path.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let fallback_dirs: Vec<PathBuf> = if let Ok(fallback) = env::var("DYLD_FALLBACK_LIBRARY_PATH")
    {
        env::split_paths(&fallback).collect()
    } else {
        let mut dirs = Vec::new();
        if let Some(home) = env::var_os("HOME") {
            dirs.push(PathBuf::from(home).join("lib"));
        }
        dirs.push(PathBuf::from("/usr/local/lib"));
        dirs.push(PathBuf::from("/usr/lib"));
        dirs
    };

    for dir in fallback_dirs {
        let candidate = dir.join(basename);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}