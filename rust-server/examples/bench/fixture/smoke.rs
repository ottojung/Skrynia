//! The benchmark guest must exercise the real v3 kernel, not old HTTP routes.
use skrynia_server::boot;
use std::fs;

struct Cleanup(std::path::PathBuf);
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn v3_benchmark_root_reads_and_writes_isolated_data() {
    let dir = std::env::temp_dir().join(format!(
        "skrynia-bench-smoke-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    fs::create_dir(&dir).unwrap();
    let _cleanup = Cleanup(dir.clone());
    fs::create_dir(dir.join("operator")).unwrap();
    fs::create_dir(dir.join("user")).unwrap();
    fs::write(
        dir.join("root.wasm"),
        wat::parse_str(include_str!("guest.wat")).unwrap(),
    )
    .unwrap();
    let kernel = boot::Kernel::start(
        &dir.join("root.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    assert_eq!(kernel.invoke(b"N").unwrap(), b"N");
    let mut body = vec![b'W'];
    body.extend([b'A'; 4096]);
    assert_eq!(kernel.invoke(&body).unwrap(), b"W");
    assert_eq!(
        fs::read(dir.join("user/bench.dat")).unwrap(),
        vec![b'A'; 4096]
    );
    assert!(!dir.join("operator/bench.dat").exists());
}
