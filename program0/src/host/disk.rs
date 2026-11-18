pub struct KernelDisk;

impl crate::cap::Disk for KernelDisk {
    type Err = i32;
    fn read_all(&self, path: &str) -> Result<Option<Vec<u8>>, i32> {
        super::read::read_file(path)
    }
    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<(), i32> {
        super::write::write_file(path, bytes)
    }
    fn remove(&self, path: &str) -> Result<(), i32> {
        let rc = unsafe { crate::root_abi::delete(path.as_ptr() as u32, path.len() as u32) };
        if rc < 0 {
            return Err(rc);
        }
        if let Some((parent, _)) = path.rsplit_once('/') {
            super::write::sync_dir(parent)?;
        }
        Ok(())
    }
    fn list(&self, dir: &str) -> Result<Vec<String>, i32> {
        super::list::list_dir(dir)
    }
}
