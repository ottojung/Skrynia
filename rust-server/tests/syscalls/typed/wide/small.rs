use super::{Discovery, kernel, start};
use libskrynia::Syscalls;

#[test]
fn small_frames_keep_the_single_byte_count() {
    let count = 254;
    let mut request = vec![1, count as u8];
    for i in 0..count {
        request.push(0x32);
        request.extend_from_slice(&(i as u32).to_le_bytes());
    }
    let list = format!(
        r#"[{{"name":"small","params":[{}],"results":["i32"]}}]"#,
        super::types(count)
    );
    let version = format!(r#"{{"version":"v","commit":"development","syscalls":{list}}}"#);
    let discovery = Discovery {
        status: 200,
        body: format!(r#"{{"functions":{list}}}"#),
        version: Some(version),
        reply: vec![1, 1, 0x32, 7, 0, 0, 0],
        request: Some(request),
    };
    let provider = start(discovery, 3);
    let import = format!(
        r#"(import "syscall" "small" (func $small {} (result i32)))"#,
        super::params(count)
    );
    let body = format!(
        "(i32.store (i32.const 0) (call $small {})) (drop (call $w (local.get $out) (i32.const 0) (i32.const 4)))",
        super::args(count)
    );
    let k = kernel(
        &super::super::root(&import, &body),
        Syscalls::http(&provider.url).unwrap(),
    )
    .unwrap();
    let output = k.invoke(b"").unwrap();
    provider.join();
    assert_eq!(output, 7i32.to_le_bytes());
}
