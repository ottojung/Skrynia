pub(super) fn check(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 128
        || name == "."
        || name == ".."
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
    {
        return Err(format!("invalid syscall name: {name:?}"));
    }
    Ok(())
}
