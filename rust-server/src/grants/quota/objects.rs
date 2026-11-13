use std::collections::BTreeMap;
const OBJECT_COST: u64 = 512;
pub fn sum(
    objects: &BTreeMap<String, u64>,
    scope: &str,
    update: Option<(&str, u64)>,
) -> (u64, u64) {
    let mut bytes = 0u64;
    let mut count = 0u64;
    let mut include = |key: &str, size: &u64| {
        bytes += update
            .filter(|(name, _)| *name == key)
            .map_or(*size, |(_, n)| n)
            + OBJECT_COST;
        count += 1;
    };
    if scope == "*" {
        for (key, size) in objects {
            include(key, size);
        }
    } else if let Some(prefix) = scope.strip_suffix('*') {
        for (key, size) in objects.range(prefix.to_string()..) {
            if !key.starts_with(prefix) {
                break;
            }
            include(key, size);
        }
    } else if let Some(size) = objects.get(scope) {
        include(scope, size);
    }
    if let Some((key, size)) = update
        && !objects.contains_key(key)
        && crate::matches(scope, key)
    {
        bytes += size + OBJECT_COST;
        count += 1;
    }
    (bytes, count)
}
#[cfg(test)]
mod tests;
