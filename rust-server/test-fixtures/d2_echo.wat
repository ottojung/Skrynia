(module
  (import "skrynia" "read" (func $read (param i32 i32 i32) (result i32)))
  (import "skrynia" "write" (func $w (param i32 i32 i32) (result i32)))
  (memory (export "memory") 4)
  (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
    (local $n i32) (local $w i32)
    (block $done (loop $again
      (local.set $n (call $read (local.get $in) (i32.const 0) (i32.const 65536)))
      (br_if $done (i32.eqz (local.get $n)))
      (if (i32.lt_s (local.get $n) (i32.const 0)) (then (return (local.get $n))))
      (local.set $w (i32.const 0))
      (block $wok (loop $wl
        (br_if $wok (i32.ge_s (local.get $w) (local.get $n)))
        (local.set $w (i32.add (local.get $w)
          (call $w (local.get $out) (local.get $w) (i32.sub (local.get $n) (local.get $w)))))
        (br $wl)))
      (br $again)))
    (i32.const 0)))
