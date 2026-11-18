(module
  (import "skrynia" "invoke"
    (func $invoke (param i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (func (export "skrynia_main")
    (param $in_ptr i32) (param $in_len i32)
    (param $out_ptr i32) (param $out_cap i32)
    (result i32)
    (i32.const 0))
)
