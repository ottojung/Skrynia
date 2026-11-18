(module
  (import "skrynia" "write" (func $w (param i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "A")
  (func (export "skrynia_main") (param i32 i32) (result i32)
    (drop (call $w (local.get 1) (i32.const 0) (i32.const 1)))
    (i32.const -5)))
