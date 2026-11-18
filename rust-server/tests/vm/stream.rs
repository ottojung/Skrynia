use skrynia_server::boot;
use std::fs;
pub(super) fn run(name: &str, root: &str, body: &[u8]) -> Result<Vec<u8>, i32> {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data")).unwrap();
    let adapted = root
        .replacen(
            "(module",
            "(module(import \"skrynia\" \"read\"(func $input_read(param i32 i32 i32)(result i32)))",
            1,
        )
        .replace("(func(export \"skrynia_main\")", "(func $old_entry");
    let i = adapted.rfind(')').unwrap();
    let wrapper = r#"(func(export "skrynia_main")(param $fd i32)(param $o i32)(param $c i32)(result i32)(local $n i32)(local.set $n(call $input_read(local.get $fd)(i32.const 32768)(i32.const 32768)))(if(result i32)(i32.lt_s(local.get $n)(i32.const 0))(then(local.get $n))(else(call $old_entry(i32.const 32768)(local.get $n)(local.get $o)(local.get $c)))))"#;
    let wrapped = format!("{}{}{}", &adapted[..i], wrapper, &adapted[i..]);
    fs::write(dir.join("r.wasm"), wat::parse_str(wrapped).unwrap()).unwrap();
    let k = boot::Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    let out = k.invoke(body);
    let _ = fs::remove_dir_all(&dir);
    out
}
