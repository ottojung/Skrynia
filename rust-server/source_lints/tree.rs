use super::{imports, lines};
use std::path::Path;

/// Enforce file size, sibling imports, and total direct directory entries
/// throughout the Rust source tree, including nested modules and tests.
pub fn visit(path: &Path, issues: &mut Vec<String>) {
    let mut entries = std::fs::read_dir(path)
        .unwrap_or_else(|e| panic!("Cannot lint {}: {e}", path.display()))
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    if entries.len() > 10 {
        issues.push(format!(
            "{}: {} direct entries exceeds 10",
            path.display(),
            entries.len()
        ));
    }
    for entry in entries {
        if entry.is_dir() {
            visit(&entry, issues);
        } else if entry.extension().is_some_and(|x| x == "rs") {
            let source = std::fs::read_to_string(&entry).unwrap();
            let count = lines::code_lines(&source);
            if count > 50 {
                issues.push(format!("{}: {count} code LOC exceeds 50", entry.display()));
            }
            imports::check(&entry, &source, issues);
        }
    }
}
