const MAX_REQ: usize = 16 * 1024 * 1024;

#[cfg(any(target_arch = "wasm32", feature = "vm-check"))]
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn skrynia_main(input_fd: i32, output_fd: i32) -> i32 {
    let req = match super::read::read_stream(input_fd, MAX_REQ) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let disk = crate::host::KernelDisk;
    let reg = match crate::reg::Registry::open(&disk, "reg") {
        Ok(v) => v,
        Err(_) => return -5,
    };
    let res = crate::handle::handle(
        &reg,
        &disk,
        &crate::host::KernelRng,
        &crate::host::KernelEval,
        &req,
    );
    let rc = super::wops::write_all(output_fd, &res);
    if rc < 0 { rc } else { 0 }
}
