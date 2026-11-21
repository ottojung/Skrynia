use std::path::PathBuf;

pub fn program() -> Result<(PathBuf, Option<&'static str>), &'static str> {
    for key in [
        "CARGO_BIN_EXE_compile_worker",
        "CARGO_BIN_EXE_skrynia_compile_worker",
    ] {
        if let Ok(p) = std::env::var(key) {
            return Ok((PathBuf::from(p), None));
        }
    }
    if let Ok(p) = std::env::var("SKRYNIA_COMPILE_WORKER") {
        return Ok((PathBuf::from(p), None));
    }
    let exe = std::env::current_exe().map_err(|_| "limit")?;
    if let Some(dir) = exe.parent() {
        for p in [dir.join("compile_worker"), dir.join("../compile_worker")] {
            if p.is_file() {
                return Ok((p, None));
            }
        }
    }
    Ok((exe, Some("--compile-worker")))
}
