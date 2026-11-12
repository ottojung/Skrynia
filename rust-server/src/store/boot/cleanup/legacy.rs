// Node.js atomicJson writes `path.<12 random bytes hex>.tmp` before rename.
pub fn generated(directory: &str, name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".tmp") else {
        return false;
    };
    let Some((target, nonce)) = stem.rsplit_once('.') else {
        return false;
    };
    if nonce.len() != 24
        || !nonce
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return false;
    }
    if directory == "state/push" {
        return target == "vapid.json";
    }
    let Some(id) = target.strip_suffix(".json") else {
        return false;
    };
    crate::valid_identifier(id)
}
#[cfg(test)]
mod tests;
