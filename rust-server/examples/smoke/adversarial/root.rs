use skrynia_server::boot::Kernel;
use std::path::Path;
pub fn check(dir: &Path) -> Result<(), String> {
    let wasm = wat::parse_str(
        r#"(module (memory (export "memory") 1)
        (func (export "skrynia_main") (param i32 i32 i32) (result i32)
        (i32.const 1048577)))"#,
    )
    .unwrap();
    let file = dir.join("root.wasm");
    std::fs::write(&file, wasm).map_err(|e| e.to_string())?;
    let k = Kernel::start(&file, &dir.join("data"))?;
    if k.invoke(&[]) != Err(-11) {
        return Err("oversized root result accepted".into());
    }
    let huge = wat::parse_str(
        r#"(module (memory (export "memory") 2048)
      (func (export "skrynia_main") (param i32 i32 i32) (result i32)
      (i32.const 0)))"#,
    )
    .unwrap();
    std::fs::write(&file, huge).map_err(|e| e.to_string())?;
    let k = Kernel::start(&file, &dir.join("data"))?;
    if k.invoke(&[]).is_ok() {
        return Err("oversized root memory instantiated".into());
    }
    Ok(())
}
