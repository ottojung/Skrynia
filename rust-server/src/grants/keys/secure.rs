pub fn match_sha(expected: &str, actual: &str) -> bool {
    if !super::valid::sha256(expected) || !super::valid::sha256(actual) {
        return false;
    }
    let diff = expected
        .bytes()
        .zip(actual.bytes())
        .fold(0u8, |d, (a, b)| d | (a ^ b));
    diff == 0
}
