use super::Value;
use super::{Run, codec::Row};
mod finish;
use std::{
    fs::{File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};
pub struct Writer {
    path: PathBuf,
    file: BufWriter<File>,
    count: u64,
    total: Value,
}
impl Writer {
    pub fn new(dir: &Path) -> io::Result<Self> {
        let path = dir.join(format!("{}.bin", crate::random_id()));
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        Ok(Self {
            path,
            file: BufWriter::new(file),
            count: 0,
            total: Value::default(),
        })
    }
    pub fn push(&mut self, key: &str, delta: Value) -> io::Result<()> {
        if delta.is_zero() {
            return Ok(());
        }
        self.total = self.total.plus(delta)?;
        let row = Row {
            key: key.into(),
            total: self.total,
        };
        self.file.write_all(&row.encode()?)?;
        self.count = self
            .count
            .checked_add(1)
            .ok_or_else(|| io::Error::other("run size overflow"))?;
        Ok(())
    }
}
