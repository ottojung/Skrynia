mod staging;
use std::fs;
use std::io;
use std::path::Path;

fn generated(name: &str) -> bool {
    let Some(id) = name.strip_prefix(".skrynia-") else {
        return false;
    };
    crate::valid_identifier(id)
}
pub fn abandoned(root: &Path) -> io::Result<()> {
    for name in [
        "state/grants",
        "state/store-transactions",
        "state/grant-revocations",
        "state/push",
        "state/push-subs",
    ] {
        let dir = root.join(name);
        let mut removed = false;
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            if file_name.to_str().is_some_and(|file| generated(file))
                && entry.file_type()?.is_file()
            {
                fs::remove_file(entry.path())?;
                removed = true;
            }
        }
        if removed {
            crate::sync_dir(&dir)?;
        }
    }
    staging::abandoned(root)?;
    Ok(())
}
#[cfg(test)]
mod tests;
