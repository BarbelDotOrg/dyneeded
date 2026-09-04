use comfy_table::{ContentArrangement, Table};
use termtree::Tree;
use crate::dependency::Dependency;

impl Dependency {
    pub fn serialize(&self, format: crate::OutputFormat) -> anyhow::Result<String> {
        match format {
            crate::OutputFormat::Ldd => Ok(self.serialize_ldd()),
            crate::OutputFormat::Text => Ok(self.serialize_text()),
            crate::OutputFormat::Json => serde_json::to_string_pretty(self)
                .map_err(|e| anyhow::anyhow!("failed to serialize to JSON: {}", e)),
            crate::OutputFormat::Ron => ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
                .map_err(|e| anyhow::anyhow!("failed to serialize to RON: {}", e)),
            crate::OutputFormat::Tree => Ok(self.serialize_tree()),
        }
    }

    fn serialize_ldd(&self) -> String {
        let mut output = String::new();
        for dep in &self.deps {
            output.push_str(&format!("{} => {}\n", dep.name, dep.path.as_ref().map_or("not found", |p| p.to_str().unwrap_or("invalid path"))));
        }
        output
    }

    fn serialize_text(&self) -> String {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Dependency", "Version", "Path"]);

        // Add root dependency
        table.add_row(vec![
            self.name.clone(),
            self.version.clone().unwrap_or_else(|| "unknown".into()),
            self.path.as_ref().and_then(|p| p.to_str()).unwrap_or("N/A").into(),
        ]);

        // Add sub-dependencies
        for dep in &self.deps {
            table.add_row(vec![
                dep.name.clone(),
                dep.version.clone().unwrap_or_else(|| "unknown".into()),
                dep.path.as_ref().and_then(|p| p.to_str()).unwrap_or("not found").into(),
            ]);
        }

        table.to_string()
    }

    fn serialize_tree(&self) -> String {
        fn build_tree(node: &Dependency) -> Tree<String> {
            let label = format!(
                "{} (version: {}) [path: {}]",
                node.name,
                node.version.as_deref().unwrap_or("unknown"),
                node.path.as_ref().and_then(|p| p.to_str()).unwrap_or("not found")
            );

            let mut tree = Tree::new(label);
            for dep in &node.deps {
                tree.push(build_tree(dep));
            }
            tree
        }

        build_tree(self).to_string()
    }
}