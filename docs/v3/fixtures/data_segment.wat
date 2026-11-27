(module
  (import "skrynia" "write1" (func $write (param i32 i32) (result i32)))
  (memory 1)
  (data (i32.const 64) "v3")
  (func (export "skrynia_main") (param i32) (param $out i32) (result i32)
    (drop (call $write (local.get $out) (i32.load8_u (i32.const 64))))
    (call $write (local.get $out) (i32.load8_u (i32.const 65))))
)
