use super::{fs, hex, load, sandbox};
use libskrynia::boot::Kernel;

#[test]
fn user_directory_fd_survives_multiple_provider_callbacks() {
    let d = sandbox("dir");
    fs::write(d.join("user/allowed/f.txt"), b"X").unwrap();
    let p1 = wat::parse_str(load("dir_p1")).unwrap();
    let root = wat::parse_str(
        load("dir_root")
            .replace("@@P1_BYTES@@", &hex(&p1))
            .replace("@@P1_LEN@@", &p1.len().to_string()),
    )
    .unwrap();
    fs::write(d.join("root.wasm"), root).unwrap();
    let k = Kernel::start(&d.join("root.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"").unwrap();
    assert_eq!(
        out, b"f.txt",
        "directory iteration must work across callbacks"
    );
    let _ = fs::remove_dir_all(d);
}
