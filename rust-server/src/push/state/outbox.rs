use crate::{Pending, open_regular};
use std::{collections::HashMap, io, path::Path};
pub fn read(root: &Path) -> io::Result<HashMap<String, Pending>> {
    let file = open_regular(&root.join("state/push/outbox.json"))?;
    let records: HashMap<String, Pending> = serde_json::from_reader(file)?;
    if records
        .iter()
        .any(|(id, pending)| !crate::valid_identifier(id) || pending.generation == 0)
    {
        return Err(io::Error::other("invalid outbox record"));
    }
    Ok(records)
}
#[cfg(test)]
mod tests;
