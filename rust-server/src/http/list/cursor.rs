use super::params::Params;
use std::{collections::BinaryHeap, fs, io, path::Path};
pub fn entries(dir: &Path, request: &Params) -> io::Result<Vec<String>> {
    let scope = &request.locations;
    if scope.is_empty() {
        return Ok(Vec::new());
    }
    let Some(prefix) = scope.strip_suffix('*') else {
        if request.after.as_ref().is_some_and(|after| after >= scope) {
            return Ok(Vec::new());
        }
        let path = dir.join("store").join(scope);
        return match crate::open_regular(&path) {
            Ok(_) => Ok(vec![scope.clone()]),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e),
        };
    };
    let mut heap = BinaryHeap::new();
    for entry in fs::read_dir(dir.join("store"))? {
        let entry = entry?;
        let key = entry
            .file_name()
            .into_string()
            .map_err(|_| io::Error::other("invalid key"))?;
        if !crate::valid_key(&key) || !entry.file_type()?.is_file() {
            return Err(io::Error::other("nonregular store entry"));
        }
        if !key.starts_with(prefix) || request.after.as_ref().is_some_and(|x| key <= *x) {
            continue;
        }
        if heap.len() < request.limit + 1 {
            heap.push(key);
        } else if heap.peek().is_some_and(|last| key < *last) {
            heap.pop();
            heap.push(key);
        }
    }
    Ok(heap.into_sorted_vec())
}
#[cfg(test)]
mod tests;
