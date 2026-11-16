use super::Value;
use super::{
    Run,
    bound::upper,
    codec::{Row, WIDTH},
};
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};
impl Run {
    fn row(file: &mut File, at: u64) -> io::Result<Row> {
        file.seek(SeekFrom::Start(at * WIDTH as u64))?;
        let mut bytes = [0u8; WIDTH];
        file.read_exact(&mut bytes)?;
        Row::decode(&bytes)
    }
    fn before(&self, file: &mut File, bound: &[u8]) -> io::Result<Value> {
        let (mut lo, mut hi) = (0, self.len);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if Self::row(file, mid)?.key.as_bytes() < bound {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo == 0 {
            Ok(Value::default())
        } else {
            Ok(Self::row(file, lo - 1)?.total)
        }
    }
    pub fn prefix(&self, prefix: &[u8]) -> io::Result<Value> {
        let mut file = File::open(&self.path)?;
        let low = self.before(&mut file, prefix)?;
        let high = match upper(prefix) {
            Some(next) => self.before(&mut file, &next)?,
            None if self.len == 0 => Value::default(),
            None => Self::row(&mut file, self.len - 1)?.total,
        };
        high.minus(low)
    }
}
