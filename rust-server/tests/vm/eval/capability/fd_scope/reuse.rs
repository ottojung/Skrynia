use super::{fs, hex, load, sandbox};
use libskrynia::boot::Kernel;

#[test]
fn authorized_fd_survives_multiple_child_callbacks() {
    let d = sandbox("reuse");
    let p1 = wat::parse_str(load("reuse_p1")).unwrap();
    let root = wat::parse_str(
        load("reuse_root")
            .replace("@@P1_BYTES@@", &hex(&p1))
            .replace("@@P1_LEN@@", &p1.len().to_string()),
    )
    .unwrap();
    fs::write(d.join("root.wasm"), root).unwrap();
    let k = Kernel::start(&d.join("root.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"").unwrap();
    assert_eq!(out.len(), 4);
    let count = i32::from_le_bytes(out.try_into().unwrap());
    assert_eq!(
        count, 1,
        "a valid child FD must survive across its callbacks"
    );
    assert_eq!(fs::read(d.join("user/allowed/f.txt")).unwrap(), b"Z");
    let _ = fs::remove_dir_all(d);
}
