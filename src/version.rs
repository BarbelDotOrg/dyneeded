use std::collections::HashMap;

pub fn version_requirements(elf: &lief::elf::Binary) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for req in elf.symbols_version_requirement() {
        let versions = req.auxiliary_symbols().map(|aux| aux.name().to_string());
        map.entry(req.name().to_string()).or_default().extend(versions);
    }
    map
}

pub fn version_key(s: &str) -> Vec<u64> {
    s.rsplit('_')
        .next()
        .unwrap_or(s)
        .split('.')
        .filter_map(|p| p.parse::<u64>().ok())
        .collect()
}

pub fn pick_version(versions: &[String]) -> Option<String> {
    versions
        .iter()
        .max_by(|a, b| version_key(a).cmp(&version_key(b)))
        .cloned()
}