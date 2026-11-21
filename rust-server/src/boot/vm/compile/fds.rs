//! Descriptor hygiene for process-isolated untrusted compilation.

#[cfg(target_os = "linux")]
fn cloexec_range() -> bool {
    unsafe {
        libc::syscall(
            libc::SYS_close_range,
            3u32,
            u32::MAX,
            libc::CLOSE_RANGE_CLOEXEC,
        ) == 0
    }
}
#[cfg(not(target_os = "linux"))]
fn cloexec_range() -> bool {
    false
}

/// Mark inherited descriptors >= 3 CLOEXEC before exec (safe in pre_exec).
pub fn mark_cloexec() {
    let _ = cloexec_range();
}

#[cfg(target_os = "linux")]
fn close_range() -> bool {
    unsafe { libc::syscall(libc::SYS_close_range, 3u32, u32::MAX, 0u32) == 0 }
}
#[cfg(not(target_os = "linux"))]
fn close_range() -> bool {
    false
}

/// Close inherited descriptors >= 3 before reading untrusted input.
pub fn close_inherited() {
    if close_range() {
        return;
    }
    let mut fds = Vec::new();
    if let Ok(dir) = std::fs::read_dir("/proc/self/fd") {
        for entry in dir.flatten() {
            if let Ok(n) = entry.file_name().to_string_lossy().parse::<i32>() {
                if n >= 3 {
                    fds.push(n);
                }
            }
        }
    }
    for fd in fds {
        unsafe {
            libc::close(fd);
        }
    }
}
