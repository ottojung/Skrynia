use super::{fs, hex, load, sandbox};
use skrynia_server::boot::Kernel;

#[test]
fn callback_owned_fd_is_disposed_after_child_trap() {
    check("trap", wat::parse_str(load("trap_p1")).unwrap());
}

#[test]
fn callback_owned_fd_is_disposed_after_child_success() {
    let p1 = wat::parse_str(
        r#"(module
      (import "skrynia" "invoke" (func $i(param i32 i32 i32 i32)(result i32)))
      (memory(export "memory") 1)
      (func (export "skrynia_main")(param i32 i32)(result i32)
        (drop (call $i (i32.const 0)(i32.const 0)(i32.const 0)(i32.const 0)))
        (i32.const 0)))"#,
    )
    .unwrap();
    check("normal", p1);
}

fn check(name: &str, p1: Vec<u8>) {
    let d = sandbox(name);
    fs::write(d.join("user/allowed/file"), b"Z").unwrap();
    let root = wat::parse_str(
        load("trap_root")
            .replace("@@P1_BYTES@@", &hex(&p1))
            .replace("@@P1_LEN@@", &p1.len().to_string()),
    )
    .unwrap();
    fs::write(d.join("root.wasm"), root).unwrap();
    let k = Kernel::start(&d.join("root.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"").unwrap();
    assert_eq!(out.len(), 4);
    let rc = i32::from_le_bytes(out.try_into().unwrap());
    assert_eq!(rc, -7, "scoped handles must close on child return or trap");
    let _ = fs::remove_dir_all(d);
}
