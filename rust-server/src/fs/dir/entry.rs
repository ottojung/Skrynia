pub struct Entry {
    pub stream: super::stream::Stream,
    pub pending: Option<Vec<u8>>,
    pub fd: std::os::fd::OwnedFd,
}
