use skrynia_server::boot::Kernel;
use std::path::Path;
pub fn check(dir: &Path) -> Result<(), String> {
    let root = wat::parse_str(
        r#"(module
       (import "skrynia" "eval" (func $eval (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
       (memory (export "memory") 1)
       (func (export "skrynia_main") (param i32 i32 i32 i32) (result i32)
         (call $eval (local.get 0) (local.get 1)
           (local.get 0) (i32.const 0) (i32.const -1)
           (local.get 2) (i32.const 2097152))))"#,
    )
    .unwrap();
    let child = wat::parse_str(
        r#"(module (memory (export "memory") 1)
       (func (export "skrynia_main") (param i32 i32 i32 i32) (result i32)
         (i32.const 1048577)))"#,
    )
    .unwrap();
    let file = dir.join("child.wasm");
    std::fs::write(&file, root).map_err(|e| e.to_string())?;
    let k = Kernel::start(&file, &dir.join("data"))?;
    if k.invoke(&child) != Err(-11) {
        return Err("oversized child result accepted".into());
    }
    Ok(())
}
