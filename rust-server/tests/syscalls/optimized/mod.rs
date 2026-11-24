mod validation;
use libskrynia::{EquivalentCall, Syscalls};

#[test]
fn optimized_matches_http_result_and_names() {
    let provider = super::provider::start(
        super::provider::Discovery {
            status: 200,
            body: r#"["double"]"#.into(),
            reply: u32::MAX.wrapping_mul(2).to_le_bytes().to_vec(),
        },
        2,
    );
    let http = Syscalls::http(&provider.url).unwrap();
    let optimized =
        Syscalls::optimized(
            [EquivalentCall::new("double", |arg| Ok(arg.wrapping_mul(2))).unwrap()],
        )
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
    provider.join();
}
