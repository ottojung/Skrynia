use super::{Discovery, kernel, start};
use libskrynia::Syscalls;

const LIST: &str = r#"[{"name":"one","params":["i32"],"results":["i32"]}]"#;
const REQUEST: [u8; 7] = [1, 1, 0x32, 1, 0, 0, 0];

#[test]
fn malformed_extended_counts_are_rejected() {
    for reply in [
        vec![1, 0xff, 1, 0, 0, 0, 0x32, 5, 0, 0, 0],
        vec![1, 0xff, 2, 0, 0, 0, 0x32, 5, 0, 0, 0, 0x32, 6, 0, 0, 0],
        vec![1, 0xff, 0, 0, 0, 0],
        vec![1, 0xff, 1, 0, 0, 0, 0x40, 5, 0, 0, 0, 0, 0, 0, 0],
        vec![1, 0xff, 1, 0],
        vec![1, 1, 0x32, 5, 0, 0, 0, 0],
    ] {
        let discovery = Discovery {
            status: 200,
            body: format!(r#"{{"functions":{LIST}}}"#),
            reply,
            request: Some(REQUEST.to_vec()),
            ..Default::default()
        };
        let provider = start(discovery, 2);
        let calls = Syscalls::http(&provider.url).unwrap();
        let wat = super::super::root(
            r#"(import "syscall" "one" (func $one (param i32) (result i32)))"#,
            "(i32.store (i32.const 0) (call $one (i32.const 1))) (drop (call $w (local.get $out) (i32.const 0) (i32.const 4)))",
        );
        let k = kernel(&wat, calls).unwrap();
        assert!(k.invoke(b"").is_err());
        provider.join();
    }
}
