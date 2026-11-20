pub(super) mod auth;
pub(super) mod concurrency;
pub(super) mod denied;
pub(super) mod recover;
pub(super) mod restart;
pub(super) mod rotate;
pub(super) mod slow;
pub(super) mod stream;
use sha2::{Digest, Sha256};
use skrynia_server::boot::Kernel;
use std::fs;
pub(super) fn root_wasm() -> Vec<u8> {
    wat::parse_str(include_str!(
        "../../../../test-fixtures/token_gated_proxy.wat"
    ))
    .unwrap()
}
pub(super) fn verifier(raw: &[u8; 32]) -> [u8; 32] {
    Sha256::digest(raw).into()
}
pub(super) fn setup(name: &str, guest: &[u8], raw: &[u8; 32]) -> (Kernel, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("skrynia-p0-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data/programs")).unwrap();
    fs::create_dir_all(dir.join("data/security")).unwrap();
    fs::write(dir.join("data/security/program0.sha256"), verifier(raw)).unwrap();
    fs::write(dir.join("data/programs/root.wasm"), guest).unwrap();
    let root = dir.join("root.wasm");
    fs::write(&root, root_wasm()).unwrap();
    (Kernel::start(&root, &dir.join("data")).unwrap(), dir)
}
pub(super) fn body(raw: &[u8; 32], payload: &[u8]) -> Vec<u8> {
    let mut b = vec![1u8];
    b.extend_from_slice(raw);
    b.extend_from_slice(payload);
    b
}
pub(super) fn echo_guest() -> Vec<u8> {
    wat::parse_str(r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(local $n i32)(local $d i32)(local $m i32)(block $e(loop $l(local.set $n(call $r(local.get $i)(i32.const 0)(i32.const 65536)))(br_if $e(i32.eqz(local.get $n)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $d(i32.const 0))(block $wok(loop $wl(br_if $wok(i32.ge_s(local.get $d)(local.get $n)))(local.set $m(call $w(local.get $o)(local.get $d)(i32.sub(local.get $n)(local.get $d))))(if(i32.lt_s(local.get $m)(i32.const 0))(then(return(local.get $m))))(local.set $d(i32.add(local.get $d)(local.get $m)))(br $wl)))(br $l)))(i32.const 0)))"#).unwrap()
}
