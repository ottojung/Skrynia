use super::root_abi as sys;

pub use super::wops::sync_dir;

pub fn write_file(path: &str, bytes: &[u8]) -> Result<(), i32> {
    let (dir, _) = path.rsplit_once('/').ok_or(-1)?;
    let dir_exists = unsafe { sys::exists(dir.as_ptr() as u32, dir.len() as u32) };
    if dir_exists < 0 {
        return Err(dir_exists);
    }
    if dir_exists == 0 {
        let rc = unsafe { sys::mkdir(dir.as_ptr() as u32, dir.len() as u32, 4) };
        if rc < 0 {
            let exists = unsafe { sys::is_dir(dir.as_ptr() as u32, dir.len() as u32) };
            if exists != 1 {
                return Err(rc);
            }
        }
    }
    let mut nonce = [0u8; 16];
    crate::cap::Random::fill(&super::rng::KernelRng, &mut nonce)?;
    let temp = format!("{}.tmp.{}", path, crate::id::to_hex(&nonce));
    let fd = unsafe { sys::open_disk(temp.as_ptr() as u32, temp.len() as u32, 14) };
    if fd < 0 {
        return Err(fd);
    }
    let status = super::wops::sync_and_close(fd, super::wops::write_all(fd, bytes));
    let result = if status == 0 {
        unsafe {
            sys::rename(
                temp.as_ptr() as u32,
                temp.len() as u32,
                path.as_ptr() as u32,
                path.len() as u32,
            )
        }
    } else {
        status
    };
    if result < 0 {
        unsafe { sys::delete(temp.as_ptr() as u32, temp.len() as u32) };
        return Err(result);
    }
    sync_dir(dir)
}
