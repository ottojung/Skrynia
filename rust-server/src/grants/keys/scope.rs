pub fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.split('/').all(|part| {
            !part.is_empty()
                && part.bytes().enumerate().all(|(i, c)| {
                    c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || (i > 0 && c == b'.')
                })
        })
}
pub fn valid_scope(loc: &str) -> bool {
    loc.len() <= 257
        && (loc == "*" || loc.is_empty() || {
            if let Some(prefix) = loc.strip_suffix('*') {
                valid_key(prefix.strip_suffix('/').unwrap_or(prefix))
            } else {
                valid_key(loc)
            }
        })
}
pub fn matches(loc: &str, key: &str) -> bool {
    if loc == "*" {
        return true;
    }
    if loc.is_empty() {
        return false;
    }
    if let Some(prefix) = loc.strip_suffix('*') {
        key.starts_with(prefix)
    } else {
        key == loc
    }
}
pub fn subset(child: &str, parent: &str) -> bool {
    if !valid_scope(child) || !valid_scope(parent) {
        return false;
    }
    if child.is_empty() {
        return true;
    }
    if parent == "*" {
        return true;
    }
    if let Some(prefix) = parent.strip_suffix('*') {
        child.starts_with(prefix)
    } else {
        child == parent
    }
}
