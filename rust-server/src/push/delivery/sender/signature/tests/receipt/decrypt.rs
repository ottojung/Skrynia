mod ecdh;
mod hkdf;
use openssl::ec::EcKey;
use openssl::error::ErrorStack;
use openssl::pkey::Private;
use openssl::symm::{Cipher, decrypt_aead};
pub fn decode(recipient: &EcKey<Private>, body: &[u8]) -> Result<Vec<u8>, ErrorStack> {
    let salt = &body[..16];
    let sender = &body[21..21 + body[20] as usize];
    let record = &body[21 + body[20] as usize..];
    let shared = ecdh::secret(recipient, sender)?;
    let recipient_public = ecdh::public(recipient)?;
    let info = [
        b"WebPush: info\0".as_slice(),
        recipient_public.as_slice(),
        sender,
    ]
    .concat();
    let ikm = hkdf::expand(&[7u8; 16], &shared, &info, 32)?;
    let key = hkdf::expand(salt, &ikm, b"Content-Encoding: aes128gcm\0", 16)?;
    let nonce = hkdf::expand(salt, &ikm, b"Content-Encoding: nonce\0", 12)?;
    let split = record.len() - 16;
    let plaintext = decrypt_aead(
        Cipher::aes_128_gcm(),
        &key,
        Some(&nonce),
        &[],
        &record[..split],
        &record[split..],
    )?;
    let end = plaintext.iter().rposition(|value| *value != 0).unwrap();
    assert_eq!(plaintext[end], 2, "valid final Web Push record");
    Ok(plaintext[..end].to_vec())
}
