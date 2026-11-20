use skrynia_server::boot::Kernel;
use std::{fs, path::PathBuf};

pub struct Fixture {
    pub path: PathBuf,
}
impl Fixture {
    pub fn new(guest: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "skrynia-p1-scope-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        for name in ["kernel", "service", "outside"] {
            fs::create_dir_all(dir.join(name)).unwrap();
        }
        let root = r#"(module
          (import "skrynia" "run_program1"(func $f(param i32 i32)(result i32)))
          (memory(export "memory") 1)
          (func(export "skrynia_main")(param i32 i32)(result i32)
            (call $f(local.get 0)(local.get 1))))"#;
        fs::write(dir.join("root.wasm"), wat::parse_str(root).unwrap()).unwrap();
        fs::write(dir.join("guest.wasm"), wat::parse_str(guest).unwrap()).unwrap();
        Self { path: dir }
    }
    pub fn start(&self, scoped: bool) -> Result<Kernel, String> {
        Kernel::start_with_scope(
            &self.path.join("root.wasm"),
            &self.path.join("kernel"),
            Some(&self.path.join("guest.wasm")),
            scoped.then(|| self.path.join("service")).as_deref(),
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
