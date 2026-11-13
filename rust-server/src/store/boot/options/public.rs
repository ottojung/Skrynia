use std::io;
// Normalize public URL paths, including dot segments.
pub fn base(input: &str) -> io::Result<String> {
    let url = url::Url::parse(input).map_err(|_| io::Error::other("invalid SKRYNIA_URL"))?;
    if !["http", "https"].contains(&url.scheme())
        || url.host().is_none()
        || url.query().is_some_and(|value| !value.is_empty())
        || url.fragment().is_some_and(|value| !value.is_empty())
    {
        return Err(io::Error::other("invalid SKRYNIA_URL"));
    }
    Ok(url.path().trim_end_matches('/').into())
}
#[cfg(test)]
mod tests;
