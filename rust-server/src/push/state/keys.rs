use crate::PushKeys;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openssl::{
    bn::BigNumContext,
    ec::{EcGroup, EcPoint},
    nid::Nid,
};
pub fn legacy(keys: &PushKeys) -> bool {
    !keys.p256dh.is_empty()
        && keys.p256dh.len() <= 256
        && !keys.auth.is_empty()
        && keys.auth.len() <= 256
}
fn decode(value: &str) -> Option<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(value).ok()
}
pub fn valid(keys: &PushKeys) -> bool {
    let Some(public) = decode(&keys.p256dh) else {
        return false;
    };
    let Some(auth) = decode(&keys.auth) else {
        return false;
    };
    if auth.len() != 16 || public.len() != 65 || public[0] != 4 {
        return false;
    }
    let Ok(group) = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1) else {
        return false;
    };
    let Ok(mut context) = BigNumContext::new() else {
        return false;
    };
    let Ok(point) = EcPoint::from_bytes(&group, &public, &mut context) else {
        return false;
    };
    point.is_on_curve(&group, &mut context).unwrap_or(false) && !point.is_infinity(&group)
}
#[cfg(test)]
mod tests;
