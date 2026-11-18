use super::support::{guest, guest_prefix, send, temp_dir};
use skrynia_program0::wire::Request;
use skrynia_server::boot::Kernel;
use std::fs;
#[test]
fn register_invoke_replace_restart_revoke() {
    let Some(root) = std::env::var_os("SKRYNIA_TEST_ROOT_WASM") else {
        return;
    };
    let dir = temp_dir();
    fs::create_dir(&dir).unwrap();
    let code = guest();
    let kernel = Kernel::start(root.as_ref(), &dir).unwrap();
    assert!(kernel.ram().is_empty());
    let initial = send(&kernel, Request::Register { code: code.clone() });
    assert_eq!(initial.status, 0);
    let id = initial.body;
    assert_eq!(id.len(), 32);
    assert_eq!(send(&kernel, Request::Lookup { id: id.clone() }).body, code);
    let input = b"hello".to_vec();
    let invoke = || Request::Invoke {
        id: id.clone(),
        input: input.clone(),
    };
    assert_eq!(send(&kernel, invoke()).body, input);
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
    assert_eq!(send(&kernel, invoke()).body, b"v2:hello");
    drop(kernel);
    let kernel = Kernel::start(root.as_ref(), &dir).unwrap();
    assert_eq!(send(&kernel, invoke()).body, b"v2:hello");
    assert_eq!(send(&kernel, Request::Revoke { id: id.clone() }).status, 0);
    assert_ne!(send(&kernel, Request::Lookup { id }).status, 0);
    assert!(kernel.ram().is_empty());
    drop(kernel);
    fs::remove_dir_all(dir).unwrap();
}
