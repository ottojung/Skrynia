#[cfg(any(target_arch = "wasm32", feature = "vm-check"))]
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn skrynia_main(in_ptr: u32, in_len: u32, out_ptr: u32, out_cap: u32) -> i32 {
    let input = unsafe { std::slice::from_raw_parts(in_ptr as *const u8, in_len as usize) };
    let disk = crate::host::KernelDisk;
    let reg = match crate::reg::Registry::open(&disk, "reg") {
        Ok(v) => v,
        Err(_) => return -5,
    };
    let result = crate::handle::handle(
        &reg,
        &disk,
        &crate::host::KernelRng,
        &crate::host::KernelEval,
        input,
    );
    if result.len() > out_cap as usize {
        return -11;
    }
    unsafe { std::ptr::copy_nonoverlapping(result.as_ptr(), out_ptr as *mut u8, result.len()) };
    result.len() as i32
}
