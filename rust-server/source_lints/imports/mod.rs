use std::path::Path;
mod head;
mod traverse;

pub fn check(path: &Path, source: &str, issues: &mut Vec<String>) {
    let directory = path.parent().unwrap();
    for (index, line) in source.lines().enumerate() {
        let statement = line.trim();
        if statement.starts_with("#[path") || statement.contains("include!(") {
            if let Some(source_path) = statement.split('"').nth(1) {
                if source_path.contains('/') || source_path.contains("..") {
                    issues.push(format!(
                        "{}:{}: nested module source",
                        path.display(),
                        index + 1
                    ));
                }
            }
        }
        let statement = statement.strip_prefix("pub ").unwrap_or(statement);
        let statement = if statement.starts_with("pub(") {
            statement
                .split_once(')')
                .map(|(_, rest)| rest.trim())
                .unwrap_or(statement)
        } else {
            statement
        };
        let Some(import) = statement.strip_prefix("use ") else {
            continue;
        };
        let import = import.trim_end_matches(';');
        traverse::inspect_import(directory, path, index, import, issues);
    }
}
