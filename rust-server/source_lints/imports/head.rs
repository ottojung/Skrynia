use std::path::{Path, PathBuf};

// The starting scope depends on how the import names its root.
pub(super) fn starting_dir(directory: &Path, head: &str) -> Option<PathBuf> {
    let module_dir = match head {
        "super" => directory.to_path_buf(),
        "self" => directory.to_path_buf(),
        "crate" => directory
            .ancestors()
            .find(|p| p.ends_with("src"))
            .unwrap_or(directory)
            .to_path_buf(),
        _ if directory.join(head).is_dir()
            || directory.join(head).with_extension("rs").exists() =>
        {
            directory.to_path_buf()
        }
        _ => return None,
    };
    Some(module_dir)
}
