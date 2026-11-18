use super::support::{guest, guest_prefix, send, temp_dir};
use skrynia_program0::wire::Request;
use skrynia_server::boot::Kernel;
#[test]
fn compiled_guest_cache_is_shared_across_requests() {
    let Some(root) = std::env::var_os("SKRYNIA_TEST_ROOT_WASM") else {
        return;
    };
    let dir = temp_dir();
    std::fs::create_dir(&dir).unwrap();
    let kernel = Kernel::start(root.as_ref(), &dir).unwrap();
    let code = guest();
    let id = send(&kernel, Request::Register { code }).body;
    let invoke = || Request::Invoke {
        id: id.clone(),
        input: b"one".to_vec(),
    };
    assert_eq!(kernel.cache().len(), 0);
    assert_eq!(send(&kernel, invoke()).body, b"one");
    assert_eq!(kernel.cache().len(), 1);
    assert_eq!(send(&kernel, invoke()).body, b"one");
    assert_eq!(kernel.cache().len(), 1);
    assert_eq!(
        send(
            &kernel,
            Request::Set {
                id: id.clone(),
                code: guest_prefix()
            }
        )
        .status,
        0
    );
    assert_eq!(send(&kernel, invoke()).body, b"v2:one");
    assert_eq!(kernel.cache().len(), 2);
    assert!(kernel.ram().is_empty());
    drop(kernel);
    std::fs::remove_dir_all(dir).unwrap();
}
