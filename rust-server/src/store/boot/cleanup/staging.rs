use std::fs;
use std::io;
use std::path::Path;
fn generated(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("obj.") else {
        return false;
    };
    let Some((pid, id)) = rest.split_once('.') else {
        return false;
    };
    !pid.is_empty()
        && pid.bytes().all(|b| b.is_ascii_digit())
        && (id.len() == 24 || id.len() == 32)
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn abandoned(root: &Path) -> io::Result<()> {
    let folder = root.join("store-tmp");
    let mut removed = false;
    for entry in fs::read_dir(&folder)? {
        let entry = entry?;
        if entry.file_type()?.is_file() && entry.file_name().to_str().is_some_and(generated) {
            fs::remove_file(entry.path())?;
            removed = true;
        }
    }
    if removed {
        crate::sync_dir(&folder)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests;
