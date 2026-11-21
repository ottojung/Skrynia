use std::process::{Command, Stdio};

pub fn spawn() -> Result<std::process::Child, &'static str> {
    let (prog, arg) = super::prog::program()?;
    let mut cmd = Command::new(prog);
    if let Some(a) = arg {
        cmd.arg(a);
    }
    cmd.env_clear().current_dir("/");
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                super::fds::mark_cloexec();
                #[cfg(target_os = "linux")]
                if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                let cpu = libc::rlimit {
                    rlim_cur: 4,
                    rlim_max: 4,
                };
                if libc::setrlimit(libc::RLIMIT_CPU, &cpu) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                let m = 512 * 1024 * 1024u64 as libc::rlim_t;
                let aslim = libc::rlimit {
                    rlim_cur: m,
                    rlim_max: m,
                };
                if libc::setrlimit(libc::RLIMIT_AS, &aslim) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    cmd.spawn().map_err(|_| "limit")
}
