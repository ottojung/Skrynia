//! Start only the disposable, explicitly configured v3 kernel.
use std::path::Path;
use std::process::{Child, Command, Stdio};
pub fn spawn(home: &Path, port: u16) -> std::io::Result<Child> {
    let current = std::env::current_exe()?;
    let executable = current
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("skrynia-server");
    Command::new(executable)
        .env("SKRYNIA_ROOT_WASM", home.join("root.wasm"))
        .env("SKRYNIA_OPERATOR_DIR", home.join("operator"))
        .env("SKRYNIA_USER_DIR", home.join("user"))
        .env("SKRYNIA_BIND", format!("127.0.0.1:{port}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}
