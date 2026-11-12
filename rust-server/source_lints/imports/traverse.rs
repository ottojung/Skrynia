use std::path::Path;
pub(super) fn inspect_import(
    directory: &Path,
    path: &Path,
    index: usize,
    import: &str,
    issues: &mut Vec<String>,
) {
    let mut parts = import.split("::").map(str::trim);
    let Some(head) = parts.next() else { return };
    let Some(mut module_dir) = super::head::starting_dir(directory, head) else {
        return;
    };
    if head == "super" && import.starts_with("super::super::") {
        issues.push(format!(
            "{}:{}: parent-of-parent import",
            path.display(),
            index + 1
        ));
        return;
    }
    let mut hops = 0;
    for part in parts {
        let item = part.trim_start_matches('{').trim_end_matches('}');
        if item.is_empty() || item.contains(',') || item.contains(' ') {
            break;
        }
        let nested = module_dir.join(item);
        if nested.is_dir() || nested.with_extension("rs").exists() {
            hops += 1;
            module_dir = nested;
        } else {
            break;
        }
    }
    if hops > 1 || head == "crate" && directory != module_dir && hops > 0 {
        issues.push(format!(
            "{}:{}: nested module import",
            path.display(),
            index + 1
        ));
    }
}
