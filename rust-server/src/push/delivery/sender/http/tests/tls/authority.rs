use super::util;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::x509::{
    X509,
    extension::{BasicConstraints, KeyUsage},
};
pub fn create() -> (PKey<Private>, X509) {
    let key = util::key();
    let name = util::name("Skrynia local test CA");
    let mut builder = X509::builder().unwrap();
    util::base(&mut builder, &key, &name, &name, 1);
    builder
        .append_extension(BasicConstraints::new().critical().ca().build().unwrap())
        .unwrap();
    builder
        .append_extension(KeyUsage::new().critical().key_cert_sign().build().unwrap())
        .unwrap();
    builder.sign(&key, MessageDigest::sha256()).unwrap();
    (key, builder.build())
}
