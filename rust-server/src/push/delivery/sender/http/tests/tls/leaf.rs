use super::util;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::x509::{
    X509,
    extension::{BasicConstraints, KeyUsage, SubjectAlternativeName},
};
pub fn create(ca_key: &PKey<Private>, ca: &X509) -> (PKey<Private>, X509) {
    let key = util::key();
    let name = util::name("push.example.org");
    let mut builder = X509::builder().unwrap();
    util::base(&mut builder, &key, &name, ca.subject_name(), 2);
    builder
        .append_extension(BasicConstraints::new().critical().build().unwrap())
        .unwrap();
    builder
        .append_extension(KeyUsage::new().digital_signature().build().unwrap())
        .unwrap();
    let san = SubjectAlternativeName::new()
        .dns("push.example.org")
        .build(&builder.x509v3_context(Some(ca), None))
        .unwrap();
    builder.append_extension(san).unwrap();
    builder.sign(ca_key, MessageDigest::sha256()).unwrap();
    (key, builder.build())
}
