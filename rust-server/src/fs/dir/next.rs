use super::Entry;

#[derive(Debug)]
pub enum Next {
    Name(usize),
    End,
    Small,
}

pub fn next_entry(e: &mut Entry, dst: &mut [u8]) -> Result<Next, &'static str> {
    let name = match e.pending.take() {
        Some(name) => name,
        None => match e.stream.next()? {
            Some(name) => name,
            None => return Ok(Next::End),
        },
    };
    if dst.len() < name.len() {
        e.pending = Some(name);
        return Ok(Next::Small);
    }
    dst[..name.len()].copy_from_slice(&name);
    Ok(Next::Name(name.len()))
}
