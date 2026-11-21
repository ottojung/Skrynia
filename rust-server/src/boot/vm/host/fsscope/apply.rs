pub fn scope(
    mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    root: i32,
    path_ptr: u32,
    path_len: u32,
    ops: u32,
) -> i32 {
    if root < 0 || ops == 0 {
        c.data_mut().fs_delegate = super::super::super::cap::FsAuth::Empty;
        return 0;
    }
    let Some(rk) = super::check::root_kind(root) else {
        return crate::boot::vm::codes::ERR_INVALID;
    };
    let mem = match crate::boot::vm::mem::get(&mut c) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let prefix = if path_len == 0 {
        Vec::new()
    } else {
        let raw = match crate::boot::vm::mem::read_buf(&mut c, mem, path_ptr, path_len) {
            Ok(x) => x,
            Err(e) => return e,
        };
        match crate::fs::path::clean(&raw) {
            Ok(p) => p.as_os_str().as_encoded_bytes().to_vec(),
            Err(e) => return crate::boot::vm::err::map(e),
        }
    };
    let cap = super::super::super::cap::FsCaps {
        root: rk,
        prefix,
        ops: ops & 0x7f,
    };
    if !super::check::allowed(&cap, &c.data().fs_auth.clone()) {
        return crate::boot::vm::codes::ERR_PERM;
    }
    c.data_mut().fs_delegate = super::super::super::cap::FsAuth::Scoped(vec![cap]);
    0
}
