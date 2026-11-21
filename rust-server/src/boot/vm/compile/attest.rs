//! Integrity envelope for worker-produced precompiled module bytes.
use sha2::{Digest, Sha256};

const MAGIC: &[u8; 8] = b"SKYRNM01";
const HEAD: usize = 8 + 32;

/// Wrap precompiled bytes with the magic header and a SHA-256 digest.
pub fn wrap(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEAD + payload.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&Sha256::digest(payload));
    out.extend_from_slice(payload);
    out
}

/// Validate the envelope, returning the payload only when it is intact.
pub fn verify(envelope: &[u8]) -> Result<&[u8], &'static str> {
    if envelope.len() < HEAD + 1 || !envelope.starts_with(MAGIC) {
        return Err("invalid");
    }
    let digest = &envelope[8..HEAD];
    let payload = &envelope[HEAD..];
    if Sha256::digest(payload).as_slice() != digest {
        return Err("invalid");
    }
    Ok(payload)
}
