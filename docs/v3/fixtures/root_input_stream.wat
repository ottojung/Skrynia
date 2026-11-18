(module
  (import "skrynia" "read" (func $read (param i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (func (export "skrynia_main")
    (param $input_fd i32) (param $out_ptr i32) (param $out_cap i32)
    (result i32)
    (call $read (local.get $input_fd) (local.get $out_ptr) (local.get $out_cap)))
)
