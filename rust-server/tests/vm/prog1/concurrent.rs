use skrynia_server::boot;
use std::{fs, sync::Arc, thread};

#[test]
fn concurrent_requests_are_isolated() {
    let d = std::env::temp_dir().join(format!("skrynia-p1cc-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("data")).unwrap();
    let root = r#"(module(import "skrynia" "run_program1"(func $r(param i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(call $r(local.get $i)(local.get $o))))"#;
    let p1 = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(local $n i32)(local $d i32)(local $x i32)(block $done(loop $l(local.set $n(call $r(local.get $i)(i32.const 0)(i32.const 4096)))(br_if $done(i32.eq(local.get $n)(i32.const 0)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $d(i32.const 0))(block $b(loop $w2(br_if $b(i32.eq(local.get $d)(local.get $n)))(local.set $x(call $w(local.get $o)(local.get $d)(i32.sub(local.get $n)(local.get $d))))(if(i32.lt_s(local.get $x)(i32.const 0))(then(return(local.get $x))))(local.set $d(i32.add(local.get $d)(local.get $x)))(br $w2)))(br $l)))(i32.const 0)))"#;
    fs::write(d.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    fs::write(d.join("p1.wasm"), wat::parse_str(p1).unwrap()).unwrap();
    let k = Arc::new(
        boot::Kernel::start_with(&d.join("r.wasm"), &d.join("data"), Some(&d.join("p1.wasm")))
            .unwrap(),
    );
    let mut hs = Vec::new();
    for t in 0..8u8 {
        let k = k.clone();
        hs.push(thread::spawn(move || {
            let body = vec![t; 64 * 1024 + t as usize];
            assert_eq!(k.invoke(&body).unwrap(), body);
        }));
    }
    for h in hs {
        h.join().unwrap();
    }
    let _ = fs::remove_dir_all(&d);
}
