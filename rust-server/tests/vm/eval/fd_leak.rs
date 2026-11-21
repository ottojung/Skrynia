//! Adversarial: the compile worker must not inherit unrelated parent fds.
#![cfg(unix)]
use std::os::fd::AsRawFd;
use std::time::{Duration, Instant};

fn worker_pids() -> Vec<u32> {
    std::fs::read_dir("/proc")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let cmd = std::fs::read(e.path().join("cmdline")).ok()?;
            let s = String::from_utf8_lossy(&cmd);
            if s.contains("compile_worker") || s.contains("--compile-worker") {
                e.file_name().to_string_lossy().parse().ok()
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn compile_worker_does_not_inherit_unrelated_descriptors() {
    let dir = std::env::temp_dir().join(format!("skrynia-fdcanary-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let canary = dir.join("canary");
    std::fs::write(&canary, b"x").unwrap();
    let file = std::fs::File::open(&canary).unwrap();
    let fd = file.as_raw_fd();
    let _ = unsafe { libc::fcntl(fd, libc::F_SETFD, 0) };
    let eng = skrynia_server::boot::vm::compile::engine().unwrap();
    let handle = std::thread::spawn(move || {
        skrynia_server::boot::vm::compile::bounded(&eng, b"SKRYNIA_HANG")
    });
    let mut leaked = false;
    let deadline = Instant::now() + Duration::from_millis(2500);
    while Instant::now() < deadline && !leaked {
        for pid in worker_pids() {
            let link = std::fs::read_link(format!("/proc/{pid}/fd/{fd}")).ok();
            if link.as_deref() == Some(canary.as_path()) {
                leaked = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let result = handle.join().unwrap();
    assert!(matches!(result, Err("limit")));
    assert!(!leaked, "compile worker inherited an unrelated descriptor");
    drop(file);
    let _ = std::fs::remove_dir_all(&dir);
}
