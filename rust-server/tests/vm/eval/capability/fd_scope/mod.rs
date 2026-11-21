mod dir;
mod nested;
mod old;
mod reuse;
mod trap;
use std::{fs, path::PathBuf};

fn sandbox(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "cap-scope-{name}-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    fs::create_dir_all(d.join("operator")).unwrap();
    fs::create_dir_all(d.join("user/allowed")).unwrap();
    d
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("\\{b:02x}")).collect()
}
fn load(name: &str) -> String {
    std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/test-fixtures/fd_scope/").to_owned() + name + ".wat",
    )
    .unwrap()
}
