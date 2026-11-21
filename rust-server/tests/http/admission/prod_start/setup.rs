use sha2::{Digest, Sha256};
use std::fs;

pub fn root_wasm() -> Vec<u8> {
    wat::parse_str(include_str!(
        "../../../../test-fixtures/token_gated_proxy.wat"
    ))
    .unwrap()
}
pub fn echo_guest() -> Vec<u8> {
    wat::parse_str(r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(local $n i32)(local $d i32)(local $m i32)(block $e(loop $l(local.set $n(call $r(local.get $i)(i32.const 0)(i32.const 65536)))(br_if $e(i32.eqz(local.get $n)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $d(i32.const 0))(block $wok(loop $wl(br_if $wok(i32.ge_s(local.get $d)(local.get $n)))(local.set $m(call $w(local.get $o)(local.get $d)(i32.sub(local.get $n)(local.get $d))))(if(i32.lt_s(local.get $m)(i32.const 0))(then(return(local.get $m))))(local.set $d(i32.add(local.get $d)(local.get $m)))(br $wl)))(br $l)))(i32.const 0)))"#).unwrap()
}
pub fn body(raw: &[u8; 32], payload: &[u8]) -> Vec<u8> {
    let mut b = vec![1u8];
    b.extend_from_slice(raw);
    b.extend_from_slice(payload);
    b
}
pub fn prepare(
    dir: &std::path::Path,
    raw: &[u8; 32],
    bad: &[u8; 32],
) -> (std::path::PathBuf, std::path::PathBuf) {
    let op = dir.join("operator");
    let user = dir.join("user");
    fs::create_dir_all(op.join("programs")).unwrap();
    fs::create_dir_all(op.join("security")).unwrap();
    fs::create_dir_all(user.join("security")).unwrap();
    fs::write(op.join("security/program0.sha256"), Sha256::digest(raw)).unwrap();
    fs::write(user.join("security/program0.sha256"), Sha256::digest(bad)).unwrap();
    fs::write(op.join("programs/root.wasm"), echo_guest()).unwrap();
    fs::write(dir.join("root.wasm"), root_wasm()).unwrap();
    (op, user)
}
