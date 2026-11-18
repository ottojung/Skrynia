use skrynia_server::boot::Kernel;
use std::path::Path;
pub fn check(dir: &Path) -> Result<(), String> {
    let children = dir.join("data/children");
    std::fs::create_dir(&children).map_err(|e| e.to_string())?;
    std::fs::write(children.join("foo"), b"ok").map_err(|e| e.to_string())?;
    let wasm = wat::parse_str(
        r#"(module
       (import "skrynia" "open_directory" (func $open (param i32 i32) (result i32)))
       (import "skrynia" "next_entry" (func $next (param i32 i32 i32) (result i32)))
       (import "skrynia" "close" (func $close (param i32) (result i32)))
       (memory (export "memory") 1)
       (data (i32.const 0) "children")
       (func (export "skrynia_main") (param i32 i32 i32) (result i32)
         (local $fd i32) (local $n i32)
         (local.set $fd (call $open (i32.const 0) (i32.const 8)))
         (drop (call $next (local.get $fd) (i32.const -1) (i32.const -1)))
         (local.set $n (call $next (local.get $fd) (local.get 1) (i32.const 16)))
         (drop (call $close (local.get $fd))) (local.get $n)))"#,
    )
    .unwrap();
    let file = dir.join("directory.wasm");
    std::fs::write(&file, wasm).map_err(|e| e.to_string())?;
    let k = Kernel::start(&file, &dir.join("data"))?;
    if k.invoke(&[]).map_err(|e| format!("directory {e}"))? != b"foo" {
        return Err("invalid buffer consumed directory entry".into());
    }
    Ok(())
}
