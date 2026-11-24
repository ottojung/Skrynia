#[test]
fn native_wins_and_accesses_ram() {
    let mut calls = libskrynia::Syscalls::new();
    calls
        .http_fallback("http://127.0.0.1:1/provider", &["double"])
        .unwrap();
    calls
        .register("double", |ram, arg| {
            let mut b = [0; 4];
            assert_eq!(ram.read(0, &mut b).unwrap(), 4);
            Ok(arg.wrapping_add(u32::from_le_bytes(b)))
        })
        .unwrap();
    let k = super::kernel(super::ROOT, calls).unwrap();
    k.ram().truncate(4).unwrap();
    k.ram().write(0, &2u32.to_le_bytes()).unwrap();
    assert_eq!(k.invoke(b"").unwrap(), 1u32.to_le_bytes());
    k.ram().write(0, &3u32.to_le_bytes()).unwrap();
    assert_eq!(k.invoke(b"").unwrap(), 2u32.to_le_bytes());
}

#[test]
fn invalid_imports_fail_at_startup() {
    assert!(super::kernel(super::ROOT, libskrynia::Syscalls::new()).is_err());
    let mut calls = libskrynia::Syscalls::new();
    calls.register("double", |_, arg| Ok(arg)).unwrap();
    assert!(calls.register("double", |_, arg| Ok(arg)).is_err());
    assert!(calls.register("../bad", |_, arg| Ok(arg)).is_err());
    let bad = super::ROOT.replace("(param i32) (result i32)))", "(param i32) (result i64)))");
    // Use a valid module with an incompatible import signature.
    let bad = bad.replace(
        "(call $double (i32.const -1))",
        "(i32.wrap_i64 (call $double (i32.const -1)))",
    );
    assert!(super::kernel(&bad, calls).is_err());
}

#[test]
fn native_failure_traps() {
    let mut calls = libskrynia::Syscalls::new();
    calls
        .register("double", |_, _| Err("provider failure".into()))
        .unwrap();
    let k = super::kernel(super::ROOT, calls).unwrap();
    assert!(k.invoke(b"").is_err());
}
