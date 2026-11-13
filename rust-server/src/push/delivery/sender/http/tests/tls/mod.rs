mod authority;
mod leaf;
mod util;
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        ServerConfig,
        pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
    },
};
pub fn bundle() -> (TlsAcceptor, reqwest::Certificate) {
    let (ca_key, ca) = authority::create();
    let (key, cert) = leaf::create(&ca_key, &ca);
    let server = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(cert.to_der().unwrap())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                key.private_key_to_pkcs8().unwrap(),
            )),
        )
        .unwrap();
    (
        TlsAcceptor::from(std::sync::Arc::new(server)),
        reqwest::Certificate::from_pem(&ca.to_pem().unwrap()).unwrap(),
    )
}
