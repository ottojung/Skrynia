use libskrynia::boot;
use std::fs;
fn start(dir: &std::path::Path) -> boot::Kernel {
    let k = boot::Kernel::start(
        &dir.join("r.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    k.ram().truncate(128).unwrap();
    k.ram().write(64, b"f").unwrap();
    k
}
#[test]
fn restart_serves_same_data_dir() {
    let root = r#"(module(import "skrynia" "read1"(func $r1(param i32)(result i32)))(func $r (param $fd i32)(param $dst i32)(param $len i32)(result i32)(local $i i32)(local $b i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $b(call $r1(local.get $fd)))(if(i32.lt_s(local.get $b)(i32.const 0))(then(if(i32.ne(local.get $b)(i32.const -1))(then(return(local.get $b))))(br $done)))(i32.store8(i32.add(local.get $dst)(local.get $i))(local.get $b))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i)))(import "skrynia" "write1"(func $w1(param i32 i32)(result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(local $r i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $r(call $w1(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(if(i32.lt_s(local.get $r)(i32.const 0))(then(return(local.get $r))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i)))(import "skrynia" "open_disk"(func $od(param i64 i64 i32)(result i32)))(import "skrynia" "close"(func $c(param i32)(result i32)))(memory(export "memory") 1)(data(i32.const 64)"f")(data(i32.const 80)"Z")(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(local $fd i32)(local $n i32)(local.set $n(call $r(local.get $i)(i32.const 0)(i32.const 1)))(if(i32.eq(i32.load8_u(i32.const 0))(i32.const 87))(then(local.set $fd(call $od (i64.const 64) (i64.const 1) (i32.const 7)))(drop(call $w(local.get $fd)(i32.const 80)(i32.const 1)))(drop(call $c(local.get $fd)))(return(i32.const 0))))(local.set $fd(call $od (i64.const 64) (i64.const 1) (i32.const 3)))(drop(call $r(local.get $fd)(i32.const 0)(i32.const 1)))(drop(call $c(local.get $fd)))(drop(call $w(local.get $o)(i32.const 0)(i32.const 1)))(i32.const 0)))"#;
    let dir = std::env::temp_dir().join(format!("skrynia-q-restart-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    start(&dir).invoke(b"W").unwrap();
    let out = start(&dir).invoke(b"R").unwrap();
    assert_eq!(out, b"Z");
    let _ = fs::remove_dir_all(dir);
}
