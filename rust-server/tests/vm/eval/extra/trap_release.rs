use skrynia_server::boot::Kernel;
use std::fs;

#[test]
fn callback_trap_releases_and_child_continues() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0)"A")(data(i32.const 16)"B")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $a i32)(local $b i32)(local.set $a(call $i(i32.const 0)(i32.const 1)(i32.const 64)(i32.const 8)))(local.set $b(call $i(i32.const 16)(i32.const 1)(i32.const 80)(i32.const 8)))(i32.store8(i32.const 32)(local.get $a))(i32.store8(i32.const 33)(local.get $b))(drop(call $w(local.get $out)(i32.const 32)(i32.const 2)))(i32.const 0)))"#).unwrap();
    let hex: String = p2.iter().map(|b| format!("\\{b:02x}")).collect();
    let p1 = format!(
        r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 1024) "{hex}")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(if(i32.eq(i32.load8_u(local.get $rp))(i32.const 65))(then(unreachable)))(i32.store8(local.get $dp)(i32.const 79))(i32.store8(i32.add(local.get $dp)(i32.const 1))(i32.const 75))(i32.const 2))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 1024)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#,
        hex = hex,
        len = p2.len()
    );
    let p1b = wat::parse_str(&p1).unwrap();
    let hex1: String = p1b.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(
        r#"(module(import "skrynia" "eval_delegate"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 4)(data(i32.const 0) "{hex}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const -1))))"#,
        hex = hex1,
        len = p1b.len()
    );
    let dir = std::env::temp_dir().join(format!("v3-trap-rel-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    let r = k.invoke(b"");
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(r.unwrap(), vec![247, 2]);
}
