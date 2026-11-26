mod native;
mod transport;
mod validate;

pub(super) fn root(import: &str, body: &str) -> String {
    format!(
        r#"(module {import}
        (import "skrynia" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))
        (func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(local $r i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $r(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(if(i32.lt_s(local.get $r)(i32.const 0))(then(return(local.get $r))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
        (memory (export "memory") 1)
        (func $sink (param $a i64) (param $b i32)
            (i64.store (i32.const 0) (local.get $a))
            (i32.store (i32.const 8) (local.get $b)))
        (func (export "skrynia_main") (param i32) (param $out i32) (result i32)
            {body}
            (i32.const 0)))"#
    )
}

pub(super) const MIX_IMPORT: &str =
    r#"(import "syscall" "mix" (func $mix (param i32 i64 i32) (result i64 i32)))"#;

pub(super) const MIX_BODY: &str = "(call $sink (call $mix (i32.const -1) (i64.const 8589934597) (i32.const 7))) (drop (call $w (local.get $out) (i32.const 0) (i32.const 12)))";

pub(super) const MIX_ARGS: [(u8, u64); 3] =
    [(0x32, u32::MAX as u64), (0x40, 8589934597), (0x32, 7)];
pub(super) const MIX_RESULTS: [(u8, u64); 2] = [(0x40, 8589934596), (0x32, 21)];
pub(super) const MIX_BYTES: [u8; 12] = [4, 0, 0, 0, 2, 0, 0, 0, 21, 0, 0, 0];
