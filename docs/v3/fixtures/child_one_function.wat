(module
  ;; An arbitrary name and integer signature supplied by the evaluator.
  (import "example" "emit" (func $emit (param i32 i64) (result i32)))
  (func (export "_start")
    (drop (call $emit (i32.const 7) (i64.const 35))))
)
