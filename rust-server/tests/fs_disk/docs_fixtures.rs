use libskrynia::boot::Kernel;
use std::fs;

const ECHO: &str = include_str!("../../../docs/v3/fixtures/root_input_stream.wat");
const CHILD: &str = include_str!("../../../docs/v3/fixtures/child_one_function.wat");
const PARENT: &str = include_str!("../../../docs/v3/fixtures/parent_callback.wat");
const PRIVATE: &str = include_str!("../../../docs/v3/fixtures/data_segment.wat");
const FORBIDDEN: &str = include_str!("../../../docs/v3/fixtures/forbidden_wasi.wat");

fn with_kernel(wat: &str, f: impl FnOnce(Result<Kernel, String>)) {
    let dir = std::env::temp_dir().join(format!(
        "skrynia-docs-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    let path = dir.join("root.wasm");
    fs::write(&path, wat::parse_str(wat).unwrap()).unwrap();
    f(Kernel::start(
        &path,
        &dir.join("operator"),
        &dir.join("user"),
    ));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn documented_scalar_root_echoes_every_byte() {
    with_kernel(ECHO, |k| {
        let bytes: Vec<u8> = (0..=255).collect();
        assert_eq!(k.unwrap().invoke(&bytes).unwrap(), bytes);
    });
}

#[test]
fn documented_single_function_eval_runs_in_parent_context() {
    with_kernel(PARENT, |k| {
        let child = wat::parse_str(CHILD).unwrap();
        assert_eq!(k.unwrap().invoke(&child).unwrap(), vec![42]);
    });
}

#[test]
fn documented_private_memory_need_not_be_exported() {
    with_kernel(PRIVATE, |k| {
        assert_eq!(k.unwrap().invoke(b"").unwrap(), b"v3")
    });
}

#[test]
fn documented_forbidden_import_fails_startup() {
    with_kernel(FORBIDDEN, |k| assert!(k.is_err()));
}
