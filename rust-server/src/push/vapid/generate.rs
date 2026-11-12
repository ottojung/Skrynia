use crate::Vapid;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openssl::bn::BigNumContext;
use openssl::ec::{EcGroup, EcKey, PointConversionForm};
use openssl::nid::Nid;
use std::io;
pub fn create() -> io::Result<Vapid> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    let key = EcKey::generate(&group)?;
    let mut ctx = BigNumContext::new()?;
    let public = key
        .public_key()
        .to_bytes(&group, PointConversionForm::UNCOMPRESSED, &mut ctx)?;
    let private = key.private_key().to_vec_padded(32)?;
    Ok(Vapid {
        public_key: URL_SAFE_NO_PAD.encode(public),
        private_key: URL_SAFE_NO_PAD.encode(private),
    })
}
