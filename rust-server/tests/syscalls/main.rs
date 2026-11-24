mod discovery;
mod optimized;
mod provider;
mod provider_serve;
mod remote;
mod version;

pub(crate) const ROOT: &str = r#"(module
    (import "syscall" "double" (func $double (param i32) (result i32)))
    (import "skrynia" "write" (func $write (param i32 i32 i32) (result i32)))
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
