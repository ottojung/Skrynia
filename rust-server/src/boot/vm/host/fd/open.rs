type Vm = crate::boot::vm::state::Vm;

#[rustfmt::skip]
fn mode(bits: u32) -> crate::fs::disk::DiskMode {
    crate::fs::disk::DiskMode { read: bits & 1 != 0, write: bits & 2 != 0, create: bits & 4 != 0,
        exclusive: bits & 8 != 0, truncate: bits & 16 != 0, }
}
#[rustfmt::skip]
fn check_mode(path: &[u8], m: crate::fs::disk::DiskMode) -> Result<Option<crate::fs::RootKind>, i32> {
    let w = m.write || m.create || m.exclusive || m.truncate;
    if !m.read && !w { return Err(crate::boot::vm::host::memory::E_NOSUP); }
    super::super::fs_root::resolve(path)
}
#[rustfmt::skip]
pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker.func_wrap("skrynia_v3_1", "open_disk", |mut c: wasmtime::Caller<'_, Vm>, ptr: u64, len: u64, m: u32| -> i32 {
        if m & !31 != 0 { return crate::boot::vm::host::memory::E_INVAL; }
        let path = match crate::boot::vm::host::memory::read(&mut c, ptr, len) { Ok(x) => x, Err(e) => return e };
        let dm = mode(m);
        let root = match check_mode(&path, dm) { Ok(r) => r, Err(e) => return e };
        let fs = c.data().fs.clone();
        match fs.open_disk(root, &mut c.data_mut().fds, &path, dm) {
            Ok(fd) => fd.0, Err(e) => crate::boot::vm::host::memory::map_fs(e),
        }
    }).map_err(|e| e.to_string())?;
    Ok(())
}
