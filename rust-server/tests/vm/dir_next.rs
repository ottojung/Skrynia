use std::fs;
#[test]
fn next_entry_reads_long_name() {
    let n = "a".repeat(255);
    let dir = std::env::temp_dir().join("v3-next-long");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator/d")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("operator/d").join(&n), b"x").unwrap();
    let root = r#"(module(import "skrynia" "open_directory"(func $od(param i32 i32)(result i32)))(import "skrynia" "next_entry"(func $ne(param i32 i32 i32)(result i32)))(import "skrynia" "close"(func $cl(param i32)(result i32)))(memory(export "memory") 2)(data(i32.const 0) "d")(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)(local $d i32)(local $n i32)(local.set $d (call $od (i32.const 0)(i32.const 1)))(local.set $n (call $ne (local.get $d)(i32.const 256)(i32.const 300)))(drop (call $cl (local.get $d)))(i32.store8 (local.get 2) (i32.eq (local.get $n)(i32.const 255)))(i32.const 1)))"#;
    let wrapped = super::adapter::adapt(root);
    fs::write(dir.join("r.wasm"), wat::parse_str(wrapped).unwrap()).unwrap();
    let k = skrynia_server::boot::Kernel::start(
        &dir.join("r.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    let out = k.invoke(b"").unwrap();
    assert_eq!(out, vec![1]);
    let _ = fs::remove_dir_all(&dir);
}
#[rustfmt::skip] #[test]
fn callback_dir_next_guard_negative_and_positive() {
    let dir = std::env::temp_dir().join(format!("v3-next-guard-{}-{}", std::process::id(), rand::random::<u64>()));
    let _ = fs::remove_dir_all(&dir); fs::create_dir_all(dir.join("operator/secret")).unwrap(); fs::create_dir_all(dir.join("user/allowed")).unwrap();
    fs::write(dir.join("operator/secret/a.txt"), b"x").unwrap(); fs::write(dir.join("user/allowed/a.txt"), b"x").unwrap();
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 0)(i32.const 64)(i32.const 64)))(i32.store(i32.const 0)(local.get $n))(drop(call $w(local.get $out)(i32.const 0)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root_neg = format!(r#"(module(import "skrynia" "open_directory"(func $od(param i32 i32)(result i32)))(import "skrynia" "next_entry"(func $ne(param i32 i32 i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_delegate"(func $ed(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 4096) "secret")(data(i32.const 4160) "allowed")(global $fd (mut i32)(i32.const -1))(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(call $ne(global.get $fd)(local.get $dp)(local.get $dc)))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(global.set $fd(call $od(i32.const 4096)(i32.const 6)))(if(i32.lt_s(global.get $fd)(i32.const 0))(then(return(global.get $fd))))(drop(call $ds(i32.const 0)(i32.const 4160)(i32.const 7)(i32.const 255)))(call $ed(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#, h=h, len=guest.len());
    fs::write(dir.join("r.wasm"), wat::parse_str(&root_neg).unwrap()).unwrap(); let k = skrynia_server::boot::Kernel::start(&dir.join("r.wasm"), &dir.join("operator"), &dir.join("user")).unwrap();
    let code = i32::from_le_bytes(k.invoke(b"").unwrap().try_into().unwrap());
    assert!(code < 0, "held root dir fd must not leak, got {code}");
    let root_pos = format!(r#"(module(import "skrynia" "open_directory"(func $od(param i32 i32)(result i32)))(import "skrynia" "next_entry"(func $ne(param i32 i32 i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_delegate"(func $ed(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 4096) "allowed")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $fd i32)(local.set $fd(call $od(i32.const 4096)(i32.const 7)))(if(i32.lt_s(local.get $fd)(i32.const 0))(then(return(local.get $fd))))(call $ne(local.get $fd)(local.get $dp)(local.get $dc)))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $ds(i32.const 0)(i32.const 4096)(i32.const 7)(i32.const 255)))(call $ed(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#, h=h, len=guest.len());
    fs::write(dir.join("r2.wasm"), wat::parse_str(&root_pos).unwrap()).unwrap();
    let k2 = skrynia_server::boot::Kernel::start(&dir.join("r2.wasm"), &dir.join("operator"), &dir.join("user")).unwrap();
    let code2 = i32::from_le_bytes(k2.invoke(b"").unwrap().try_into().unwrap());
    assert!(code2 > 0, "scoped dir next should succeed, got {code2}"); let _ = fs::remove_dir_all(&dir);
}
#[rustfmt::skip] #[test]
fn callback_can_cas_own_ram_fd() {
    let d = std::env::temp_dir().join(format!("v3-cas-ok-{}-{}", std::process::id(), rand::random::<u64>()));
    let _ = fs::remove_dir_all(&d); fs::create_dir_all(d.join("operator")).unwrap(); fs::create_dir_all(d.join("user")).unwrap();
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 0)(i32.const 64)(i32.const 64)))(i32.store(i32.const 0)(local.get $n))(drop(call $w(local.get $out)(i32.const 0)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(r#"(module(import "skrynia" "open_ram"(func $or(result i32)))(import "skrynia" "truncate"(func $tr(param i32 i64)(result i32)))(import "skrynia" "cas_ram_u64"(func $cas(param i32 i64 i64 i64 i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_delegate"(func $ed(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 4096) "allowed")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $fd i32)(local.set $fd(call $or))(drop(call $tr(local.get $fd)(i64.const 8)))(call $cas(local.get $fd)(i64.const 0)(i64.const 0)(i64.const 1)(i32.const 128)))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $ds(i32.const 0)(i32.const 4096)(i32.const 7)(i32.const 255)))(call $ed(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#, h=h, len=guest.len());
    fs::write(d.join("r.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = skrynia_server::boot::Kernel::start(&d.join("r.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"").unwrap(); assert_eq!(i32::from_le_bytes(out.try_into().unwrap()), 0); let _ = fs::remove_dir_all(d);
}
