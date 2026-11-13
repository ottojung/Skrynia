#[derive(Debug)]
pub enum Error {
    Ignore,
    Unsatisfiable,
}
pub fn parse(header: &str, size: u64) -> Result<(u64, u64), Error> {
    let value = header.strip_prefix("bytes=").ok_or(Error::Ignore)?;
    if value.contains(',') {
        return Err(Error::Ignore);
    }
    let (a, b) = value.split_once('-').ok_or(Error::Ignore)?;
    if a.is_empty() && b.is_empty() || b.contains('-') {
        return Err(Error::Ignore);
    }
    if a.is_empty() {
        let suffix: u64 = b.parse().map_err(|_| Error::Ignore)?;
        if suffix == 0 || size == 0 {
            return Err(Error::Unsatisfiable);
        }
        return Ok((size.saturating_sub(suffix), size - 1));
    }
    let start: u64 = a.parse().map_err(|_| Error::Ignore)?;
    let end: u64 = if b.is_empty() {
        size.saturating_sub(1)
    } else {
        b.parse().map_err(|_| Error::Ignore)?
    };
    if start >= size || end < start {
        return Err(Error::Unsatisfiable);
    }
    Ok((start, end.min(size - 1)))
}
