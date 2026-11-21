#[cfg(unix)]
pub fn close_inherited() -> Result<(), std::io::Error> {
    #[cfg(target_os = "linux")]
    {
        // Close all non-stdio fds in the child after fork/exec setup.
        // Prefer close_range (async-signal-safe). Fall back to bounded loop.
        let ret = libc::syscall(libc::SYS_close_range, 3u32, !0u32, 0u32);
        if ret == 0 {
            return Ok(());
        }
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() != Some(libc::ENOSYS) && err.raw_os_error() != Some(libc::EINVAL) {
            // Some kernels/configs may return other errors; fall through.
        }
    }
    let mut fd: libc::c_int = 3;
    while fd <= 4096 {
        // Best-effort; ignore EBADF and continue.
        unsafe {
            libc::close(fd);
        }
        fd += 1;
    }
    Ok(())
}
