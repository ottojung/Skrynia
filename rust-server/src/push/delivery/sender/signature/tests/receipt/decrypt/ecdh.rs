use openssl::bn::BigNumContext;
use openssl::derive::Deriver;
use openssl::ec::{EcGroup, EcKey, EcPoint, PointConversionForm};
use openssl::error::ErrorStack;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
pub fn public(key: &EcKey<Private>) -> Result<Vec<u8>, ErrorStack> {
    let mut ctx = BigNumContext::new()?;
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    key.public_key()
        .to_bytes(&group, PointConversionForm::UNCOMPRESSED, &mut ctx)
}
pub fn secret(recipient: &EcKey<Private>, sender: &[u8]) -> Result<Vec<u8>, ErrorStack> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    let mut ctx = BigNumContext::new()?;
    let point = EcPoint::from_bytes(&group, sender, &mut ctx)?;
    let sender = PKey::from_ec_key(EcKey::from_public_key(&group, &point)?)?;
    let recipient = PKey::from_ec_key(recipient.to_owned())?;
    let mut deriver = Deriver::new(&recipient)?;
    deriver.set_peer(&sender)?;
    deriver.derive_to_vec()
}
