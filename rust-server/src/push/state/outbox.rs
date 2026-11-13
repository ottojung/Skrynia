use crate::{Notification, Pending, open_regular};
use std::{collections::HashMap, io, path::Path};
pub fn read(root: &Path) -> io::Result<HashMap<String, Notification>> {
    let file = open_regular(&root.join("state/push/outbox.json"))?;
    let records: HashMap<String, Pending> = serde_json::from_reader(file)?;
    records
        .into_iter()
        .map(|(id, pending)| {
            if !crate::valid_identifier(&id) || pending.generation == 0 {
                return Err(io::Error::other("invalid outbox record"));
            }
            Ok((id.clone(), pending.into_notice(id)))
        })
        .collect()
}

#[cfg(test)]
mod tests;
