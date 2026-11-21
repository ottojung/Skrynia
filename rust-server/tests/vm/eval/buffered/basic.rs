use skrynia_server::boot::Kernel;
use std::fs;

pub fn run(name: &str, root: &str) -> Result<Vec<u8>, i32> {
    let d = std::env::temp_dir().join(format!("sk-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("data")).unwrap();
    fs::write(d.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = Kernel::start(&d.join("r.wasm"), &d.join("data")).unwrap();
    let r = k.invoke(b"");
    let _ = fs::remove_dir_all(&d);
    r
}
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("\\{b:02x}")).collect()
}

#[test]
fn buffered_eval_zero_small_roundtrip() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 64) "PONG")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 16)))(if(i32.ne(local.get $n)(i32.const 4))(then(return(i32.const -5))))(if(i32.ne(i32.load(i32.const 0))(i32.const 0x474e4950))(then(return(i32.const -6))))(drop(call $w(local.get $out)(i32.const 64)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h = hex(&p2);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 3)(data(i32.const 0) "{h}")(data(i32.const 512) "PING")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 0)(i32.const {len})(i32.const 512)(i32.const 4)(i32.const 768)(i32.const 8)(i32.const -1)))(if(i32.ne(local.get $n)(i32.const 4))(then(return(local.get $n))))(if(i32.ne(i32.load(i32.const 768))(i32.const 0x474e4f50))(then(return(i32.const -7))))(i32.const 0)))"#,
        h = h,
        len = p2.len()
    );
    assert_eq!(run("buf-basic", &root).unwrap(), Vec::<u8>::new());
}
