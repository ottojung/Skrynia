use super::{fs, hex, load, sandbox};
use libskrynia::boot::Kernel;

#[test]
fn direct_nested_eval_cannot_retarget_private_operator_fd() {
    let d = sandbox("nested");
    fs::write(d.join("operator/secret.txt"), b"TOPSECRET").unwrap();
    let p2 = wat::parse_str(load("nested_p2")).unwrap();
    let p1 = wat::parse_str(
        load("nested_p1")
            .replace("@@P2_BYTES@@", &hex(&p2))
            .replace("@@P2_LEN@@", &p2.len().to_string()),
    )
    .unwrap();
    let root = wat::parse_str(
        load("nested_root")
            .replace("@@P1_BYTES@@", &hex(&p1))
            .replace("@@P1_LEN@@", &p1.len().to_string()),
    )
    .unwrap();
    fs::write(d.join("root.wasm"), root).unwrap();
    let k = Kernel::start(&d.join("root.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"hello").unwrap();
    assert_ne!(
        out, b"TOPSECRET",
        "nested eval must not reauthorize operator FD"
    );
    let _ = fs::remove_dir_all(d);
}
