const PREFIX: &[u8] = b"SKRYNIA_FD_PROBE\n";

pub fn code(buf: &[u8]) -> Option<&[u8]> {
    if cfg!(debug_assertions) && buf.starts_with(PREFIX) {
        if leaked() {
            return None;
        }
        return Some(&buf[PREFIX.len()..]);
    }
    Some(buf)
}

#[cfg(all(debug_assertions, unix, target_os = "linux"))]
fn leaked() -> bool {
    let mut st = std::mem::MaybeUninit::<libc::stat>::uninit();
    for fd in 3..512 {
        if unsafe { libc::fstat(fd, st.as_mut_ptr()) } == 0 {
            return true;
        }
    }
    false
}

#[cfg(not(all(debug_assertions, unix, target_os = "linux")))]
fn leaked() -> bool {
    false
}
