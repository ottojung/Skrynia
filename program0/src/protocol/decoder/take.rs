#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeErr {
    Short,
    Version,
    Op,
}
pub fn take<'a>(src: &mut &'a [u8], n: usize) -> Result<&'a [u8], DecodeErr> {
    if src.len() < n {
        return Err(DecodeErr::Short);
    }
    let (head, tail) = src.split_at(n);
    *src = tail;
    Ok(head)
}
pub fn id(src: &mut &[u8]) -> Result<Vec<u8>, DecodeErr> {
    let n = u16::from_le_bytes(take(src, 2)?.try_into().unwrap()) as usize;
    Ok(take(src, n)?.to_vec())
}
pub fn blob(src: &mut &[u8]) -> Result<Vec<u8>, DecodeErr> {
    let n = u32::from_le_bytes(take(src, 4)?.try_into().unwrap()) as usize;
    Ok(take(src, n)?.to_vec())
}
