#[cfg(all(unix, target_os = "linux"))]
mod fd;

use std::time::{Duration, Instant};

fn workers() -> usize {
    std::fs::read_dir("/proc")
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .chars()
                .all(|c| c.is_ascii_digit())
        })
        .filter(|e| {
            std::fs::read(e.path().join("cmdline"))
                .map(|b| {
                    let s = String::from_utf8_lossy(&b);
                    s.contains("compile_worker") || s.contains("--compile-worker")
                })
                .unwrap_or(false)
        })
        .count()
}

#[test]
fn compile_worker_cannot_inherit_parent_fds() {
    let eng = skrynia_server::boot::vm::compile::engine().unwrap();
    let wasm = wat::parse_str("(module)").unwrap();
    assert!(skrynia_server::boot::vm::compile::bounded(&eng, &wasm).is_ok());
    #[cfg(all(unix, target_os = "linux"))]
    {
        let _secret = fd::InheritedFd::open_secret();
        let mut probe = b"SKRYNIA_FD_PROBE\n".to_vec();
        probe.extend_from_slice(&wasm);
        let t = Instant::now();
        let r = skrynia_server::boot::vm::compile::bounded(&eng, &probe);
        assert!(t.elapsed() < Duration::from_millis(4200));
        assert!(r.is_ok(), "compile worker inherited parent fd/secret");
    }
    let deadline = Instant::now() + Duration::from_millis(800);
    while Instant::now() < deadline && workers() != 0 {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(workers(), 0);
    assert!(skrynia_server::boot::vm::compile::bounded(&eng, &wasm).is_ok());
}
