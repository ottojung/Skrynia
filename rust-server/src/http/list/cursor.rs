use super::params::Params;
use std::{collections::BinaryHeap, io, path::Path};
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
    let mut heap = BinaryHeap::<String>::new();
    crate::store::paths::visit(dir, |key, _| {
        if !key.starts_with(prefix) || request.after.as_ref().is_some_and(|x| key <= x.as_str()) {
            return Ok(());
        }
        if heap.len() < request.limit + 1 {
            heap.push(key.into());
        } else if heap.peek().is_some_and(|last| key < last.as_str()) {
            heap.pop();
            heap.push(key.into());
        }
        Ok(())
    })?;
    Ok(heap.into_sorted_vec())
}
#[cfg(test)]
mod tests;
