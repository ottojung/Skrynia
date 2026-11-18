pub struct RamFd {
    pub pos: u64,
}

pub enum Item {
    File(crate::fs::disk::Entry),
    Dir(crate::fs::dir::Entry),
    Ram(RamFd),
}
