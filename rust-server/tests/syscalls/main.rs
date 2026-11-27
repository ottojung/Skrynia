mod discovery;
mod optimized;
mod provider;
mod provider_serve;
mod remote;
mod typed;
mod version;

pub(crate) const ROOT: &str = r#"(module
    (import "syscall" "double" (func $double (param i32) (result i32)))
    (import "skrynia" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $write (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(local $r i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $r(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(if(i32.lt_s(local.get $r)(i32.const 0))(then(return(local.get $r))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
    (memory (export "memory") 1)
    (func (export "skrynia_main") (param i32) (param $out i32) (result i32)
        (i32.store (i32.const 0) (call $double (i32.const -1)))
        (drop (call $write (local.get $out) (i32.const 0) (i32.const 4)))
        (i32.const 0)))"#;

pub(crate) fn kernel(
    wat: &str,
    syscalls: libskrynia::Syscalls,
) -> Result<libskrynia::boot::Kernel, String> {
    let dir = std::env::temp_dir().join(format!(
        "skrynia-syscall-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    std::fs::create_dir_all(dir.join("op")).unwrap();
    std::fs::create_dir_all(dir.join("user")).unwrap();
    std::fs::write(dir.join("root.wasm"), wat::parse_str(wat).unwrap()).unwrap();
    let result = libskrynia::boot::Kernel::start_with_syscalls(
        &dir.join("root.wasm"),
        &dir.join("op"),
        &dir.join("user"),
        syscalls,
    );
    // The tests use only volatile RAM and open root directory descriptors.
    std::fs::remove_dir_all(dir).unwrap();
    result
}
