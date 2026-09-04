use crate::dependency::Dependency;
use comfy_table::{Cell, Color as TableColor, ContentArrangement, Table};
use owo_colors::{OwoColorize, Stream::Stdout, Style};
use termtree::Tree;

impl Dependency {
    pub fn serialize(&self, format: crate::OutputFormat) -> anyhow::Result<String> {
        match format {
            crate::OutputFormat::Ldd => Ok(self.serialize_ldd()),
            crate::OutputFormat::Text => Ok(self.serialize_text()),
            crate::OutputFormat::Json => serde_json::to_string_pretty(self)
                .map_err(|e| anyhow::anyhow!("failed to serialize to JSON: {}", e)),
            crate::OutputFormat::Ron => {
                ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
                    .map_err(|e| anyhow::anyhow!("failed to serialize to RON: {}", e))
            }
            crate::OutputFormat::Tree => Ok(self.serialize_tree()),
        }
    }

    fn serialize_ldd(&self) -> String {
        let mut output = String::new();
        for dep in &self.deps {
            let found = dep.path.is_some();
            let path_str = dep.path.as_ref().map_or("not found".to_string(), |p| {
                p.to_str().unwrap_or("invalid path").to_string()
            });

            let name = dep.name.if_supports_color(Stdout, |t| t.cyan()).to_string();
            let path = if found {
                path_str
                    .if_supports_color(Stdout, |t| t.green())
                    .to_string()
            } else {
                path_str.if_supports_color(Stdout, |t| t.red()).to_string()
            };

            output.push_str(&format!("{} => {}\n", name, path));
        }
        output
    }

    fn serialize_text(&self) -> String {
        let mut table = Table::new();
        table
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Dependency", "Version", "Path"]);

        table.add_row(Self::row(
            &self.name,
            self.version.as_deref(),
            self.path.as_ref(),
        ));
        for dep in &self.deps {
            table.add_row(Self::row(
                &dep.name,
                dep.version.as_deref(),
                dep.path.as_ref(),
            ));
        }

        table.to_string()
    }

    fn row(name: &str, version: Option<&str>, path: Option<&std::path::PathBuf>) -> Vec<Cell> {
        let colorize = supports_color();

        let version_str = version.unwrap_or("unknown").to_string();
        let (path_str, found) = match path.and_then(|p| p.to_str()) {
            Some(p) => (p.to_string(), true),
            None => ("not found".to_string(), false),
        };

        let mut name_cell = Cell::new(name);
        let mut version_cell = Cell::new(&version_str);
        let mut path_cell = Cell::new(&path_str);

        if colorize {
            name_cell = name_cell.fg(TableColor::Cyan);
            version_cell = version_cell.fg(if version.is_some() {
                TableColor::Yellow
            } else {
                TableColor::DarkGrey
            });
            path_cell = path_cell.fg(if found {
                TableColor::Green
            } else {
                TableColor::Red
            });
        }

        vec![name_cell, version_cell, path_cell]
    }

    fn serialize_tree(&self) -> String {
        fn build_tree(node: &Dependency) -> Tree<String> {
            let name_style = Style::new().cyan().bold();
            let name = node
                .name
                .if_supports_color(Stdout, |t| t.style(name_style))
                .to_string();

            let version_style = Style::new().yellow();
            let version = node
                .version
                .as_deref()
                .unwrap_or("unknown")
                .if_supports_color(Stdout, |t| t.style(version_style))
                .to_string();

            let found = node.path.is_some();
            let path_str = node
                .path
                .as_ref()
                .and_then(|p| p.to_str())
                .unwrap_or("not found")
                .to_string();
            let path_style = if found {
                Style::new().green()
            } else {
                Style::new().red()
            };
            let path = path_str
                .if_supports_color(Stdout, |t| t.style(path_style))
                .to_string();

            let label = format!("{name} (version: {version}) [path: {path}]");
            let mut tree = Tree::new(label);
            for dep in &node.deps {
                tree.push(build_tree(dep));
            }
            tree
        }
        build_tree(self).to_string()
    }
}

fn supports_color() -> bool {
    use std::io::IsTerminal;
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    std::io::stdout().is_terminal()
}
