use std::io;
pub fn key() -> io::Result<usize> {
    let length = std::env::var("SKRYNIA_MAX_KEY_LENGTH")
        .ok()
        .map_or(Ok(256usize), |value| value.parse::<usize>())
        .map_err(|_| io::Error::other("invalid key length"))?;
    if length == 0 || length > 9_007_199_254_740_991usize {
        return Err(io::Error::other("invalid key length"));
    }
    Ok(length)
}
