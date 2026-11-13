use crate::{Notification, Pending, write_json};
use std::{
    collections::{BTreeMap, HashMap},
    io,
    path::Path,
};
pub fn save(root: &Path, records: &HashMap<String, Notification>) -> io::Result<()> {
    let stored = records
        .iter()
        .map(|(id, entry)| (id.as_str(), Pending::from_notice(entry)))
        .collect::<BTreeMap<_, _>>();
    let result = write_json(&root.join("state/push/outbox.json"), &stored);
    #[cfg(test)]
    if result.is_ok() {
        WRITES.with(|value| value.set(value.get() + 1));
    }
    result
}

#[cfg(test)]
thread_local! {
    static WRITES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
pub fn count_writes() -> usize {
    WRITES.with(std::cell::Cell::get)
}
