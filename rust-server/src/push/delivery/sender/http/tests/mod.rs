mod fixture;
mod server;
mod tls;
#[tokio::test]
async fn encrypted_push_crosses_verified_https_with_status_handling() {
    for (status, expected) in [
        (201, Ok(())),
        (410, Err(super::super::Failure::Gone)),
        (302, Err(super::super::Failure::Retry)),
    ] {
        let (address, root, thread) = server::start(status).await;
        let (sub, config, vapid, path) = fixture::sample(address.port());
        let msg = super::super::signature::create(&sub, &config, &vapid).unwrap();
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .add_root_certificate(root)
            .resolve("push.example.org", address)
            .timeout(std::time::Duration::from_secs(4))
            .build()
            .unwrap();
        assert_eq!(super::post(&client, &sub, msg).await, expected);
        thread.await.unwrap();
        std::fs::remove_dir_all(path).unwrap();
    }
}

#[tokio::test]
async fn untrusted_https_certificate_is_rejected() {
    let (address, _root, server) = server::start(201).await;
    let (sub, config, vapid, path) = fixture::sample(address.port());
    let message = super::super::signature::create(&sub, &config, &vapid).unwrap();
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .resolve("push.example.org", address)
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();
    assert_eq!(
        super::post(&client, &sub, message).await,
        Err(super::super::Failure::Retry)
    );
    server.abort();
    std::fs::remove_dir_all(path).unwrap();
}
