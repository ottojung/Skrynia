#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(in_ptr: u32, in_len: u32, out_ptr: u32, out_cap: u32) -> i32 {
    let _ = (in_ptr, in_len, out_ptr, out_cap);
    -1
}
