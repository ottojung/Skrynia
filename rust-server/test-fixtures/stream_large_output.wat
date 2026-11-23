(module
  (import "skrynia" "write" (func $w (param i32 i32 i32) (result i32)))
  (memory (export "memory") 2)
  (data (i32.const 0) "L")
  (func (export "skrynia_main") (param i32 i32) (result i32)
    (local $i i32)
    (memory.fill (i32.const 0) (i32.const 76) (i32.const 16384))
    (loop $again
      (drop (call $w (local.get 1) (i32.const 0) (i32.const 16384)))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br_if $again (i32.lt_u (local.get $i) (i32.const 128))))
    (i32.const 0)))
