use std::fs::File;

pub struct Entry {
    pub file: File,
    pub can_read: bool,
    pub can_write: bool,
}
