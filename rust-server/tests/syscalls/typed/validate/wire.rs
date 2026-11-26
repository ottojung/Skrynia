use libskrynia::Syscalls;

#[test]
fn malformed_wire_reply_is_rejected() {
    for reply in [
        vec![],
        vec![1, 2],
        vec![2, 2],
        vec![1, 1, b'l'],
        super::super::super::provider::wire(&super::super::MIX_RESULTS[..1]),
        {
            let mut bytes = super::super::super::provider::wire(&super::super::MIX_RESULTS);
            bytes.push(0);
            bytes
        },
    ] {
        let discovery = super::super::super::provider::Discovery {
            status: 200,
            body: super::super::transport::LIST.into(),
            reply,
            request: Some(super::super::super::provider::wire(&super::super::MIX_ARGS)),
            ..Default::default()
        };
        let provider = super::super::super::provider::start(discovery, 2);
        let calls = Syscalls::http(&provider.url).unwrap();
        let wat = super::super::root(super::super::MIX_IMPORT, super::super::MIX_BODY);
        let k = super::super::super::kernel(&wat, calls).unwrap();
        assert!(k.invoke(b"").is_err());
        provider.join();
    }
}
