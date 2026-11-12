pub fn parse(bytes: &[u8]) -> Option<(usize, usize)> {
    let end = bytes.windows(4).position(|w| w == b"\r\n\r\n")?;
    let header = String::from_utf8_lossy(&bytes[..end]).to_ascii_lowercase();
    let length = header
        .lines()
        .find_map(|line| line.strip_prefix("content-length: "))?
        .parse::<usize>()
        .ok()?;
    Some((end, length))
}
