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
fn compile_timeout_clean_failure_and_responsive() {
    let eng = skrynia_server::boot::vm::compile::engine().unwrap();
    let t = Instant::now();
    let r = skrynia_server::boot::vm::compile::bounded(&eng, b"SKRYNIA_HANG");
    assert!(t.elapsed() < Duration::from_millis(900));
    assert!(matches!(r, Err("limit")));
    let deadline = Instant::now() + Duration::from_millis(800);
    while Instant::now() < deadline && workers() != 0 {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(workers(), 0);
    let wasm = wat::parse_str("(module)").unwrap();
    assert!(skrynia_server::boot::vm::compile::bounded(&eng, &wasm).is_ok());
    let bad = skrynia_server::boot::vm::compile::bounded(&eng, b"\0asm-bad");
    assert!(matches!(bad, Err("invalid")));
}
