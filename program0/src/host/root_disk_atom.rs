#[cfg(target_arch = "wasm32")]
use super::{id, root_abi as host};

#[cfg(target_arch = "wasm32")]
pub fn write_atomic(path: &str, bytes: &[u8]) -> Result<(), i32> {
    let (dir, _) = path.rsplit_once('/').ok_or(-1)?;
    super::root_disk_ls::ensure(dir)?;
    let mut id = [0u8; 16];
    let rc = unsafe { host::secure_random_bits(id.as_mut_ptr() as u32, 128) };
    if rc < 0 {
        return Err(rc);
    }
    let tmp = format!("{dir}/.{}.tmp", id::to_hex(&id));
    let fd = unsafe { host::open_disk(tmp.as_ptr() as u32, tmp.len() as u32, 15) };
    if fd < 0 {
        return Err(fd);
    }
    let written = super::root_disk_io::write_bytes(fd, bytes);
    let closed = unsafe { host::close(fd) };
    if let Err(e) = written {
        unsafe { host::delete(tmp.as_ptr() as u32, tmp.len() as u32) };
        return Err(e);
    }
    if closed < 0 {
        return Err(closed);
    }
    let n = unsafe {
        host::rename(
            tmp.as_ptr() as u32,
            tmp.len() as u32,
            path.as_ptr() as u32,
            path.len() as u32,
        )
    };
    if n < 0 {
        return Err(n);
    }
    let dfd = unsafe { host::open_directory(dir.as_ptr() as u32, dir.len() as u32) };
    if dfd < 0 {
        return Err(dfd);
    }
    let sync = unsafe { host::sync(dfd) };
    unsafe { host::close(dfd) };
    if sync < 0 { Err(sync) } else { Ok(()) }
}
