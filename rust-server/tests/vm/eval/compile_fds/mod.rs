#[cfg(all(unix, target_os = "linux"))]
mod fd;

use std::time::{Duration, Instant};

#[test]
fn compile_worker_cannot_inherit_parent_fds() {
    let eng = skrynia_server::boot::vm::compile::engine().unwrap();
    let wasm = wat::parse_str("(module)").unwrap();
    assert!(skrynia_server::boot::vm::compile::bounded(&eng, &wasm).is_ok());
    #[cfg(all(unix, target_os = "linux"))]
    {
        // Adversarial: parent holds an inheritable fd (secret/root handle).
        // Debug worker probe fails compile if any non-stdio fd leaked.
        let _secret = fd::InheritedFd::open_secret();
        let mut probe = b"SKRYNIA_FD_PROBE\n".to_vec();
        probe.extend_from_slice(&wasm);
        let t = Instant::now();
        let r = skrynia_server::boot::vm::compile::bounded(&eng, &probe);
        assert!(t.elapsed() < Duration::from_millis(4200));
        assert!(r.is_ok(), "compile worker inherited parent fd/secret");
    }
    // Worker remains kill/reap-safe and responsive after adversarial probe.
    assert!(skrynia_server::boot::vm::compile::bounded(&eng, &wasm).is_ok());
    let bad = skrynia_server::boot::vm::compile::bounded(&eng, b"\0asm-bad");
    assert!(matches!(bad, Err("invalid")));
}
