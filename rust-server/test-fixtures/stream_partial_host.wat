(module
  (import "skrynia_v3_1" "read1" (func $sk_read1_1 (param i32) (result i32)))
  (import "skrynia_v3_1" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))
  (func $r (param $fd i32)(param $dst i32)(param $len i32)(result i32)(local $i i32)(local $b i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $b(call $sk_read1_1(local.get $fd)))(if(i32.lt_s(local.get $b)(i32.const 0))(then(if(i32.ne(local.get $b)(i32.const -1))(then(return(local.get $b))))(br $done)))(i32.store8(i32.add(local.get $dst)(local.get $i))(local.get $b))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
  (func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(local $r i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $r(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(if(i32.lt_s(local.get $r)(i32.const 0))(then(return(local.get $r))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
  (memory (export "memory") 3)
  ;; Target byte-I/O contract: read1/write1 transfer one byte, there is no
  ;; bulk MAX_IO limit and no -8 short-transfer status. Read the whole request
  ;; byte-for-byte, confirm EOF, then echo every byte back.
  (func (export "skrynia_main") (param $in i32) (param $out i32)
    (result i32)
    (if (i32.ne (call $r (local.get $in) (i32.const 0)
        (i32.const 131072)) (i32.const 131072))
      (then (return (i32.const -9))))
    (if (i32.ne (call $r (local.get $in) (i32.const 0)
        (i32.const 1)) (i32.const 0))
      (then (return (i32.const -10))))
    (if (i32.ne (call $w (local.get $out) (i32.const 0)
        (i32.const 131072)) (i32.const 131072))
      (then (return (i32.const -11))))
    (i32.const 0)))
