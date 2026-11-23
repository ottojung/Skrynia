use std::process::Command;

pub(super) const BANNED: &str = "Web Push|WebPush|root grant|issuer credential|issuerToken|X-Skrynia-If-SHA256|GET /store|PUT /store|SKRYNIA_URL|STORE_DIR|/grants/|GET /version|GET /health|quota|grant|subscribe|notification";
pub(super) const INV: &str =
    "MUST NOT implement grants|The only endpoint is `POST /`|SKRYNIA_ROOT_WASM|ERR_PERM";
pub(super) const TAGS: &str = "`docs/v3-vm-spec.md`|README|";
pub(super) const BASE: &str = "8c0c14655584a7dead3c7df5ce490d23e59f9242";

pub(super) fn git(a: &[&str]) -> String {
    let output = Command::new("git")
        .args(a)
        .output()
        .expect("git executable");
    assert!(
        output.status.success(),
        "git {a:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

pub(super) fn ids(s: &str) -> Vec<&str> {
    s.split("$id-").skip(1).map(|c| &c[..16]).collect()
}
