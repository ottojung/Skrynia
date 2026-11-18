use super::Entry;

#[derive(Debug)]
pub enum Next {
    Name(usize),
    End,
    Small,
}

pub fn next_entry(e: &mut Entry, dst: &mut [u8]) -> Result<Next, &'static str> {
    if e.pos >= e.names.len() {
        return Ok(Next::End);
    }
    let name = e.names[e.pos].as_bytes();
    if dst.len() < name.len() {
        return Ok(Next::Small);
    }
    dst[..name.len()].copy_from_slice(name);
    e.pos += 1;
    Ok(Next::Name(name.len()))
}
