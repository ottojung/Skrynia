use std::process::{Command, Stdio};

pub fn spawn() -> Result<std::process::Child, &'static str> {
    let (prog, arg) = super::prog::program()?;
    let mut cmd = Command::new(prog);
    if let Some(a) = arg {
        cmd.arg(a);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if std::env::var_os("SKRYNIA_COMPILE_WORKER_HANG").is_some() {
        cmd.env("SKRYNIA_COMPILE_WORKER_HANG", "1");
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                let cpu = libc::rlimit {
                    rlim_cur: 1,
                    rlim_max: 1,
                };
                let _ = libc::setrlimit(libc::RLIMIT_CPU, &cpu);
                let m = 512 * 1024 * 1024u64 as libc::rlim_t;
                let aslim = libc::rlimit {
                    rlim_cur: m,
                    rlim_max: m,
                };
                let _ = libc::setrlimit(libc::RLIMIT_AS, &aslim);
                Ok(())
            });
        }
    }
    cmd.spawn().map_err(|_| "limit")
}
