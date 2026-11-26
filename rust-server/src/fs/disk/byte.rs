use super::Entry;
use std::io::{Read, Seek, SeekFrom, Write};

pub fn read1(e: &mut Entry) -> Result<Option<u8>, &'static str> {
    if !e.can_read {
        return Err("perm");
    }
    let mut b = [0u8; 1];
    match e.file.read(&mut b) {
        Ok(0) => Ok(None),
        Ok(_) => Ok(Some(b[0])),
        Err(_) => Err("io"),
    }
}

pub fn write1(e: &mut Entry, byte: u8) -> Result<(), &'static str> {
    if !e.can_write {
        return Err("perm");
    }
    e.file.write_all(&[byte]).map_err(|_| "io")
}

pub fn seek_rel(e: &mut Entry, shift: i64) -> Result<u64, &'static str> {
    let cur = e.file.stream_position().map_err(|_| "io")?;
    let len = e.file.metadata().map_err(|_| "io")?.len();
    let next = i128::from(cur) + i128::from(shift);
    if next < 0 || next > i64::MAX as i128 || next as u64 > len {
        return Err("overflow");
    }
    e.file
        .seek(SeekFrom::Start(next as u64))
        .map_err(|_| "io")?;
    Ok(next as u64)
}
