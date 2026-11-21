use std::path::Path;

fn canonical_dir(path: &Path) -> Result<std::path::PathBuf, String> {
    std::fs::canonicalize(path).map_err(|e| format!("canonicalize data dir: {e}"))
}

pub fn ensure_distinct(
    operator: &crate::fs::root::Root,
    user: &crate::fs::root::Root,
    operator_path: &Path,
    user_path: &Path,
) -> Result<(), String> {
    if operator.same_fs(user) {
        return Err("operator and user roots must be physically distinct".into());
    }
    let op = canonical_dir(operator_path)?;
    let us = canonical_dir(user_path)?;
    if op == us || op.starts_with(&us) || us.starts_with(&op) {
        return Err("operator and user roots must be disjoint".into());
    }
    if super::ancestry::is_ancestor(operator, user)?
        || super::ancestry::is_ancestor(user, operator)?
    {
        return Err("operator and user roots must be disjoint".into());
    }
    Ok(())
}
