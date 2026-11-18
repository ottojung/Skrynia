use std::sync::atomic::{AtomicU64, Ordering};
static SEQ: AtomicU64 = AtomicU64::new(0);
pub fn temp_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "skrynia-v3-e2e-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ))
}
use skrynia_program0::wire::{Request, Response};
use skrynia_program0::{wdec, wenc};
use skrynia_server::boot::Kernel;
pub fn send(kernel: &Kernel, req: Request) -> Response {
    let bytes = wenc::encode_req(&req);
    let response = kernel.invoke(&bytes).expect("VM accepted request");
    wdec::decode_res(&response).expect("valid response")
}
pub fn guest() -> Vec<u8> {
    wat::parse_str(
        r#"(module(memory(export "memory") 1)
    (func(export "skrynia_main")(param $i i32)(param $n i32)
      (param $o i32)(param $c i32)(result i32)
      (if (i32.gt_u (local.get $n) (local.get $c))
          (then (return (i32.const -11))))
      (memory.copy(local.get $o)(local.get $i)(local.get $n))
      (local.get $n)))"#,
    )
    .unwrap()
}

pub fn guest_prefix() -> Vec<u8> {
    wat::parse_str(
        r#"(module(memory(export "memory") 1)
       (data (i32.const 1024) "v2:")
       (func(export "skrynia_main")(param $i i32)(param $n i32)
         (param $o i32)(param $c i32)(result i32)
         (if (i32.gt_u (i32.add (local.get $n) (i32.const 3)) (local.get $c))
           (then (return (i32.const -11))))
         (memory.copy (local.get $o) (i32.const 1024) (i32.const 3))
         (memory.copy (i32.add (local.get $o) (i32.const 3))
                      (local.get $i) (local.get $n))
         (i32.add (local.get $n) (i32.const 3))))"#,
    )
    .unwrap()
}
