use skrynia_server::boot::Kernel;
use std::fs;

pub fn run(name: &str, root: &str) -> Result<Vec<u8>, i32> {
    let d = std::env::temp_dir().join(format!(
        "sk-{name}-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("operator")).unwrap();
    fs::create_dir_all(d.join("user")).unwrap();
    fs::write(d.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = Kernel::start(&d.join("r.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let r = k.invoke(b"");
    let _ = fs::remove_dir_all(&d);
    r
}
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("\\{b:02x}")).collect()
}
