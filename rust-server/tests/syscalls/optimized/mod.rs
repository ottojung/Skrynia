mod validation;
use libskrynia::{EquivalentCall, Syscalls};
use wasmtime::Val;

#[test]
fn optimized_matches_http_result_and_names() {
    let reply = super::provider::wire(&[(0x32, u32::MAX.wrapping_mul(2) as u64)]);
    let provider = super::provider::start(
        super::provider::Discovery {
            status: 200,
            body: super::provider::DOUBLE.into(),
            reply,
            request: Some(super::provider::wire(&[(0x32, u32::MAX as u64)])),
            ..Default::default()
        },
        3,
    );
    let http = Syscalls::http(&provider.url).unwrap();
    let optimized = Syscalls::optimized([EquivalentCall::new("double", "i", "i", |args| {
        Ok(vec![Val::I32(args[0].unwrap_i32().wrapping_mul(2))])
    })
    .unwrap()])
    .unwrap();
    assert_eq!(http.names(), optimized.names());
    let reference = super::kernel(super::ROOT, http)
        .unwrap()
        .invoke(b"")
        .unwrap();
    let equivalent = super::kernel(super::ROOT, optimized)
        .unwrap()
        .invoke(b"")
        .unwrap();
    assert_eq!(reference, equivalent);
    assert_eq!(equivalent, vec![254, 255, 255, 255]);
    provider.join();
}
