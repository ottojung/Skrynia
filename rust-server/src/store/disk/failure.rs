use std::{error::Error, fmt, io};
#[derive(Debug)]
pub struct AtomicFailure {
    pub installed: bool,
    pub errno: Option<i32>,
    pub cause: io::Error,
}
impl fmt::Display for AtomicFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cause.fmt(f)
    }
}
impl Error for AtomicFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}
pub fn annotate(error: io::Error, installed: bool) -> io::Error {
    io::Error::new(
        error.kind(),
        AtomicFailure {
            installed,
            errno: error.raw_os_error(),
            cause: error,
        },
    )
}
pub fn clean(error: &io::Error) -> bool {
    error
        .get_ref()
        .and_then(|e| e.downcast_ref::<AtomicFailure>())
        .is_some_and(|e| !e.installed)
}
pub fn errno(error: &io::Error) -> Option<i32> {
    error.raw_os_error().or_else(|| {
        error
            .get_ref()
            .and_then(|e| e.downcast_ref::<AtomicFailure>())
            .and_then(|e| e.errno)
    })
}
