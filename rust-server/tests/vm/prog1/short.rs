use skrynia_server::boot;
use std::fs;

#[test]
fn short_reads_and_writes_stream_correctly() {
    let d = std::env::temp_dir().join(format!("skrynia-p1short-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("data")).unwrap();
    let root = r#"(module(import "skrynia" "run_program1"(func $r(param i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(call $r(local.get $i)(local.get $o))))"#;
    let p1 = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(local $n i32)(local $x i32)(block $d(loop $l(local.set $n(call $r(local.get $i)(i32.const 0)(i32.const 1)))(br_if $d(i32.eq(local.get $n)(i32.const 0)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $x(call $w(local.get $o)(i32.const 0)(local.get $n)))(if(i32.lt_s(local.get $x)(i32.const 0))(then(return(local.get $x))))(br $l)))(i32.const 0)))"#;
    fs::write(d.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    fs::write(d.join("p1.wasm"), wat::parse_str(p1).unwrap()).unwrap();
    let k = boot::Kernel::start_with(&d.join("r.wasm"), &d.join("data"), Some(&d.join("p1.wasm")))
        .unwrap();
    let body: Vec<u8> = (0..5000).map(|i| (i % 251) as u8).collect();
    assert_eq!(k.invoke(&body).unwrap(), body);
    let _ = fs::remove_dir_all(&d);
}
