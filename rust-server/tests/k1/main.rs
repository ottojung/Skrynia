mod byte_io;
mod input_seek;
mod root_dir;
mod shared_ram;

use libskrynia::boot::Kernel;
use std::fs;
use std::path::PathBuf;

pub fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "k1-{name}-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("operator")).unwrap();
    fs::create_dir_all(d.join("user")).unwrap();
    d
}
pub fn kernel(d: &PathBuf, wat: &str) -> Kernel {
    let p = d.join("root.wasm");
    fs::write(&p, wat::parse_str(wat).unwrap()).unwrap();
    Kernel::start(&p, &d.join("operator"), &d.join("user")).unwrap()
}
