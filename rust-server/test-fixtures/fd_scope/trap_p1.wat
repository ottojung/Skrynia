(module
      (import "skrynia" "invoke" (func $i(param i32 i32 i32 i32)(result i32)))
      (memory(export "memory") 1)
      (func (export "skrynia_main")(param i32 i32)(result i32)
        (drop (call $i (i32.const 0)(i32.const 0)(i32.const 0)(i32.const 0)))
        (unreachable)))
