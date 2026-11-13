use crate::Vapid;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openssl::bn::{BigNum, BigNumContext};
use openssl::ec::{EcGroup, EcPoint, PointConversionForm};
use openssl::nid::Nid;
use std::io;
pub fn check(value: &Vapid) -> io::Result<()> {
    let secret = URL_SAFE_NO_PAD
        .decode(&value.private_key)
        .map_err(|_| io::Error::other("invalid VAPID private key"))?;
    let public = URL_SAFE_NO_PAD
        .decode(&value.public_key)
        .map_err(|_| io::Error::other("invalid VAPID public key"))?;
    if secret.len() != 32 || public.len() != 65 || public[0] != 4 {
        return Err(io::Error::other("invalid VAPID key size"));
    }
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    let mut ctx = BigNumContext::new()?;
    let mut point = EcPoint::new(&group)?;
    let number = BigNum::from_slice(&secret)?;
    point.mul_generator2(&group, &number, &mut ctx)?;
    if point.to_bytes(&group, PointConversionForm::UNCOMPRESSED, &mut ctx)? != public {
        return Err(io::Error::other("invalid VAPID keypair"));
    }
    Ok(())
}
