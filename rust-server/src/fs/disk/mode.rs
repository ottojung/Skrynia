#[derive(Clone, Copy, Debug)]
pub struct DiskMode {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub exclusive: bool,
    pub truncate: bool,
}

impl DiskMode {
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            create: false,
            exclusive: false,
            truncate: false,
        }
    }
    pub fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            create: true,
            exclusive: false,
            truncate: false,
        }
    }
    pub fn create_new_rw() -> Self {
        Self {
            read: true,
            write: true,
            create: true,
            exclusive: true,
            truncate: false,
        }
    }
}
