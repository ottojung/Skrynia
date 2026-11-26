use libskrynia::Syscalls;

pub(super) const LIST: &str =
    r#"[{"name":"mix","params":["i32","i64","i32"],"results":["i64","i32"]}]"#;

#[test]
fn typed_transport_matches_native_for_mixed_widths() {
    let version = format!(r#"{{"version":"v","commit":"development","syscalls":{LIST}}}"#);
    let discovery = super::super::provider::Discovery {
        status: 200,
        body: format!(r#"{{"functions":{LIST}}}"#),
        version: Some(version),
        reply: super::super::provider::wire(&super::MIX_RESULTS),
        request: Some(super::super::provider::wire(&super::MIX_ARGS)),
    };
    let provider = super::super::provider::start(discovery, 3);
    let http = Syscalls::http(&provider.url).unwrap();
    let native = Syscalls::optimized([super::native::mix()]).unwrap();
    let wat = super::root(super::MIX_IMPORT, super::MIX_BODY);
    let transported = super::super::kernel(&wat, http)
        .unwrap()
        .invoke(b"")
        .unwrap();
    let compiled = super::super::kernel(&wat, native)
        .unwrap()
        .invoke(b"")
        .unwrap();
    provider.join();
    assert_eq!(transported, super::MIX_BYTES);
    assert_eq!(transported, compiled);
}
