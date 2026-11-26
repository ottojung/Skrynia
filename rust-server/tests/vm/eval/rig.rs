use libskrynia::boot::Kernel;
use std::fs;
use std::path::PathBuf;

pub struct Rig {
    pub dir: PathBuf,
    pub kernel: Kernel,
}

impl Drop for Rig {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

pub fn rig(name: &str, root: &str) -> Rig {
    let dir = std::env::temp_dir().join(format!(
        "k2-eval-{name}-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("root.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let kernel = Kernel::start(
        &dir.join("root.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    Rig { dir, kernel }
}

pub fn put(r: &Rig, offset: u64, bytes: &[u8]) {
    r.kernel
        .ram()
        .truncate(offset as usize + bytes.len())
        .unwrap();
    r.kernel.ram().write(offset as usize, bytes).unwrap();
}
