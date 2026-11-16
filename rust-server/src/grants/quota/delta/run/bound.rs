pub(super) fn upper(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut next = prefix.to_vec();
    for i in (0..next.len()).rev() {
        if next[i] < u8::MAX {
            next[i] += 1;
            next.truncate(i + 1);
            return Some(next);
        }
    }
    None
}
