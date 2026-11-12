mod generate;
mod validate;
use crate::{Vapid, write_json};
use std::fs;
use std::io;
use std::path::Path;
pub fn load_vapid(root: &Path) -> io::Result<Vapid> {
    let file = root.join("state/push/vapid.json");
    if file.exists() {
        let value: Vapid = serde_json::from_slice(&fs::read(&file)?)?;
        validate::check(&value)?;
        return Ok(value);
    }
    let value = generate::create()?;
    write_json(&file, &value)?;
    Ok(value)
}

#[cfg(test)]
mod tests;
