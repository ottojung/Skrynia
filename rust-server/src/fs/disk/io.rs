use super::Entry;
use std::io::{Read, Seek, SeekFrom, Write};

pub fn read(e: &mut Entry, dst: &mut [u8]) -> Result<usize, &'static str> {
    if !e.can_read {
        return Err("perm");
    }
    e.file.read(dst).map_err(|_| "io")
}
pub fn write(e: &mut Entry, src: &[u8]) -> Result<usize, &'static str> {
    if !e.can_write {
        return Err("perm");
    }
    e.file.write(src).map_err(|_| "io")
}
pub fn seek(e: &mut Entry, off: u64) -> Result<(), &'static str> {
    e.file
        .seek(SeekFrom::Start(off))
        .map_err(|_| "io")
        .map(|_| ())
}
pub fn truncate(e: &Entry, size: u64) -> Result<(), &'static str> {
    if !e.can_write {
        return Err("perm");
    }
    e.file.set_len(size).map_err(|_| "io")
}
pub fn sync(e: &Entry) -> Result<(), &'static str> {
    e.file.sync_all().map_err(|_| "io")
}
