use super::codec::WIDTH;
use sha2::{Digest, Sha256};
use std::io;
const BODY: usize = WIDTH - 32;
pub fn stamp(bytes: &mut [u8; WIDTH]) {
    let hash = Sha256::digest(&bytes[..BODY]);
    bytes[BODY..].copy_from_slice(&hash);
}
pub fn verify(bytes: &[u8; WIDTH]) -> io::Result<()> {
    if Sha256::digest(&bytes[..BODY]).as_slice() != &bytes[BODY..] {
        return Err(io::Error::other("derived quota index checksum mismatch"));
    }
    Ok(())
}
