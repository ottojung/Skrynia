#[derive(Clone, Copy, Debug)]
pub struct GuestCap {
    pub a: crate::fs::Fd,
    pub b: crate::fs::Fd,
}

impl GuestCap {
    pub fn allows(&self, fd: crate::fs::Fd) -> bool {
        fd == self.a || fd == self.b
    }
}
