use skrynia_server::boot::Kernel;
use std::fs;
fn run(name: &str, root: &str) -> Result<Vec<u8>, i32> {
    let d = std::env::temp_dir().join(format!("sk-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("operator")).unwrap();
    fs::create_dir_all(d.join("user")).unwrap();
    fs::write(d.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = Kernel::start(&d.join("r.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let r = k.invoke(b"");
    let _ = fs::remove_dir_all(&d);
    r
}
fn data(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("\\{b:02x}")).collect()
}
#[test]
fn mediator_allows_and_denies() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0)"ALLOW")(data(i32.const 16)"DENY")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $a i32)(local $d i32)(local.set $a(call $i(i32.const 0)(i32.const 5)(i32.const 64)(i32.const 8)))(local.set $d(call $i(i32.const 16)(i32.const 4)(i32.const 80)(i32.const 8)))(i32.store8(i32.const 32)(i32.eq(local.get $a)(i32.const 2)))(i32.store8(i32.const 33)(i32.eq(local.get $d)(i32.const -4)))(drop(call $w(local.get $out)(i32.const 32)(i32.const 2)))(i32.const 0)))"#).unwrap();
    let p1 = format!(
        r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 1024) "{}")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(if(i32.ge_u(local.get $rl)(i32.const 5))(then(if(i32.eq(i32.load8_u(local.get $rp))(i32.const 65))(then(i32.store8(local.get $dp)(i32.const 79))(i32.store8(i32.add(local.get $dp)(i32.const 1))(i32.const 75))(return(i32.const 2))))))(i32.const -4))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 1024)(i32.const {})(local.get $in)(local.get $out)(i32.const 0))))"#,
        data(&p2),
        p2.len()
    );
    let p1b = wat::parse_str(&p1).unwrap();
    let root = format!(
        r#"(module(import "skrynia" "eval_delegate"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 4)(data(i32.const 0) "{}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {})(local.get $in)(local.get $out)(i32.const -1))))"#,
        data(&p1b),
        p1b.len()
    );
    assert_eq!(run("med-ok", &root).unwrap(), vec![1, 1]);
}
