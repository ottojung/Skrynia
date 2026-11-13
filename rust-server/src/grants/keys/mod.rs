mod hash;
mod scope;
mod secure;
mod valid;
pub use hash::{sha, sha_file};
use rand::RngCore;
pub use scope::{matches, subset, valid_key, valid_scope};
pub fn random_id() -> String {
    let mut bytes = [0; 16];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
pub fn secret() -> String {
    let mut bytes = [0; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[cfg(test)]
mod tests;

pub use valid::{identifier as valid_identifier, sha256 as valid_sha256};

pub use secure::match_sha as secure_match;
