use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Signer;
fn mac(key: &[u8], bytes: &[u8]) -> Result<Vec<u8>, ErrorStack> {
    let key = PKey::hmac(key)?;
    let mut signer = Signer::new(MessageDigest::sha256(), &key)?;
    signer.update(bytes)?;
    signer.sign_to_vec()
}
pub fn expand(
    salt: &[u8],
    input: &[u8],
    info: &[u8],
    length: usize,
) -> Result<Vec<u8>, ErrorStack> {
    let prk = mac(salt, input)?;
    let data = [info, &[1]].concat();
    Ok(mac(&prk, &data)?[..length].to_vec())
}
