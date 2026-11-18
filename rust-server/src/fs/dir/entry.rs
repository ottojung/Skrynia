pub struct Entry {
    pub names: Vec<String>,
    pub pos: usize,
    pub fd: std::os::fd::OwnedFd,
}
