use super::{Value, check};
use std::io;
pub const WIDTH: usize = 2 + 255 + 16 + 16 + 32;
pub struct Row {
    pub key: String,
    pub total: Value,
}
impl Row {
    pub fn encode(&self) -> io::Result<[u8; WIDTH]> {
        let size = self.key.len();
        if size > 255 || !crate::valid_key(&self.key) {
            return Err(io::Error::other("invalid indexed key"));
        }
        let mut bytes = [0u8; WIDTH];
        bytes[..2].copy_from_slice(&(size as u16).to_le_bytes());
        bytes[2..2 + size].copy_from_slice(self.key.as_bytes());
        bytes[257..273].copy_from_slice(&self.total.objects.to_le_bytes());
        bytes[273..289].copy_from_slice(&self.total.bytes.to_le_bytes());
        check::stamp(&mut bytes);
        Ok(bytes)
    }
    pub fn decode(data: &[u8; WIDTH]) -> io::Result<Self> {
        check::verify(data)?;
        let size = usize::from(u16::from_le_bytes([data[0], data[1]]));
        if size > 255 {
            return Err(io::Error::other("invalid indexed key length"));
        }
        let key = String::from_utf8(data[2..2 + size].to_vec())
            .map_err(|_| io::Error::other("invalid index encoding"))?;
        if !crate::valid_key(&key) {
            return Err(io::Error::other("invalid indexed key"));
        }
        let objects = i128::from_le_bytes(data[257..273].try_into().unwrap());
        let bytes = i128::from_le_bytes(data[273..289].try_into().unwrap());
        Ok(Self {
            key,
            total: Value { objects, bytes },
        })
    }
}
