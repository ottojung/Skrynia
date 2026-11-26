use super::{Discovery, kernel, start};
use libskrynia::Syscalls;

#[test]
fn extended_count_is_used_in_both_http_directions() {
    let args: Vec<u64> = (0..super::N as u64).collect();
    let results: Vec<u64> = args.iter().map(|v| v * 2).collect();
    let list = format!(
        r#"[{{"name":"wide","params":[{}],"results":[{}]}}]"#,
        super::types(super::N),
        super::types(super::N)
    );
    let version = format!(r#"{{"version":"v","commit":"development","syscalls":{list}}}"#);
    let discovery = Discovery {
        status: 200,
        body: format!(r#"{{"functions":{list}}}"#),
        version: Some(version),
        reply: super::frame(&results),
        request: Some(super::frame(&args)),
    };
    let provider = start(discovery, 3);
    let import = format!(
        r#"(import "syscall" "wide" (func $wide {} {}))"#,
        super::params(super::N),
        super::results(super::N)
    );
    let body = format!(
        "(call $wide {}) {} (drop (call $w (local.get $out) (i32.const 0) (i32.const {})))",
        super::args(super::N),
        super::stores(super::N),
        super::N * 4
    );
    let module = super::super::root_with(&super::put(), &import, &body);
    let k = kernel(&module, Syscalls::http(&provider.url).unwrap()).unwrap();
    let output = k.invoke(b"").unwrap();
    provider.join();
    assert_eq!(output, super::bytes(&results));
}
