use std::io;
pub fn key() -> io::Result<usize> {
    let length = std::env::var("SKRYNIA_MAX_KEY_LENGTH")
        .ok()
        .map_or(Ok(255usize), |value| value.parse::<usize>())
        .map_err(|_| io::Error::other("invalid key length"))?;
    if length == 0 || length > 255 {
        return Err(io::Error::other("invalid key length"));
    }
    Ok(length)
}
