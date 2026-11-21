use super::{fs, hex, load, sandbox};
use skrynia_server::boot::Kernel;

#[test]
fn callback_owned_fd_is_disposed_after_child_trap() {
    let d = sandbox("trap");
    fs::write(d.join("user/allowed/file"), b"Z").unwrap();
    let p1 = wat::parse_str(load("trap_p1")).unwrap();
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
    assert_eq!(rc, -7, "scoped handles must close on child trap");
    let _ = fs::remove_dir_all(d);
}
