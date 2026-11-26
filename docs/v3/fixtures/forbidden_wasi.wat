(module
  (import "wasi_snapshot_preview1" "fd_write"
    (func (param i32 i32 i32 i32) (result i32)))
  (func (export "skrynia_main") (param i32 i32) (result i32)
    (i32.const 0))
)
