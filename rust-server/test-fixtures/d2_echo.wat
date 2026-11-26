(module
  (import "skrynia" "read1" (func $sk_read1_1 (param i32) (result i32)))(func $read (param $fd i32)(param $dst i32)(param $len i32)(result i32)(local $i i32)(local $b i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $b(call $sk_read1_1(local.get $fd)))(br_if $done(i32.lt_s(local.get $b)(i32.const 0)))(i32.store8(i32.add(local.get $dst)(local.get $i))(local.get $b))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
  (import "skrynia" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(drop(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $len))
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
