use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openssl::bn::BigNumContext;
use openssl::ec::{EcGroup, EcKey, PointConversionForm};
use openssl::nid::Nid;
use serde_json::{Value, json};
pub fn subscription(endpoint: &str) -> Value {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let key = EcKey::generate(&group).unwrap();
    let mut context = BigNumContext::new().unwrap();
    let public = key
        .public_key()
        .to_bytes(&group, PointConversionForm::UNCOMPRESSED, &mut context)
        .unwrap();
    json!({"locations":"notice.*","events":["create","replace","delete"],
        "endpoint":endpoint,"keys":{"p256dh":URL_SAFE_NO_PAD.encode(public),
        "auth":URL_SAFE_NO_PAD.encode([7u8;16])}})
}
