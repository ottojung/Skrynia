#[cfg(target_arch = "wasm32")]
use super::cap::Disk;
#[cfg(target_arch = "wasm32")]
use super::root_abi as host;

#[cfg(target_arch = "wasm32")]
pub struct HostDisk;

#[cfg(target_arch = "wasm32")]
impl Disk for HostDisk {
    type Err = i32;

    fn read_all(&self, path: &str) -> Result<Option<Vec<u8>>, i32> {
        super::root_disk_io::read_all(path)
    }
    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<(), i32> {
        super::root_disk_atom::write_atomic(path, bytes)
    }
    fn remove(&self, path: &str) -> Result<(), i32> {
        let n = unsafe { host::delete(path.as_ptr() as u32, path.len() as u32) };
        if n < 0 { Err(n) } else { Ok(()) }
    }
    fn list(&self, dir: &str) -> Result<Vec<String>, i32> {
        super::root_disk_ls::list(dir)
    }
}
